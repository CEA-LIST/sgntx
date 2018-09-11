//
//   (C) Copyright 2017 CEA LIST. All Rights Reserved.
//   Contributor(s): Thibaud Tortech & Sergiu Carpov
//
//   This software is governed by the CeCILL-C license under French law and
//   abiding by the rules of distribution of free software.  You can  use,
//   modify and/ or redistribute the software under the terms of the CeCILL-C
//   license as circulated by CEA, CNRS and INRIA at the following URL
//   "http://www.cecill.info".
//
//   As a counterpart to the access to the source code and  rights to copy,
//   modify and redistribute granted by the license, users are provided only
//   with a limited warranty  and the software's author,  the holder of the
//   economic rights,  and the successive licensors  have only  limited
//   liability.
//
//   The fact that you are presently reading this means that you have had
//   knowledge of the CeCILL-C license and that you accept its terms.
//


use alloc::boxed::Box;
use alloc::vec::Vec;

use core::u64;
use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, AtomicU8, ATOMIC_U8_INIT, Ordering};
use core::ops::{Drop, Deref, DerefMut};

use sgx_types::{sgx_spinlock_t, sgx_spin_lock, sgx_spin_unlock, SGX_SPINLOCK_INITIALIZER};
use sgx_trts::trts::rsgx_read_rand;

use core::fmt::Display;
use shared::{Hash, as_u8_slice_mut};

use console;
use core::mem;


fn rsgx_spin_lock(lock: &mut sgx_spinlock_t) {
    unsafe { sgx_spin_lock( lock ); }
}

fn rsgx_spin_unlock(lock: &mut sgx_spinlock_t) {
    unsafe { sgx_spin_unlock( lock ); }
}


struct Bucket<K,V> {
    tag:   AtomicU8,
    lock:  sgx_spinlock_t,
    key:   K,
    value: V,
}

pub struct Guard<'a, V: ?Sized + 'a>
{
    lock: &'a mut sgx_spinlock_t,
    data: &'a mut V,
}


impl<K,V> Bucket<K,V> {
    pub fn is_used(&self) -> bool {
        self.tag.load(Ordering::Relaxed) != 0
    }
    
    pub fn clear(&mut self) {
        self.tag = ATOMIC_U8_INIT;
    }

    pub fn lock(&mut self) -> Guard<V>
    {
        rsgx_spin_lock( &mut self.lock );

        Guard {
            lock: &mut self.lock,
            data: &mut self.value,
        }
    }
}


impl<'a, V: ?Sized> Deref for Guard<'a, V>
{
    type Target = V;
    fn deref<'b>(&'b self) -> &'b V { &*self.data }
}


impl<'a, V: ?Sized> DerefMut for Guard<'a, V>
{
    fn deref_mut<'b>(&'b mut self) -> &'b mut V { &mut *self.data }
}


impl<'a, V: ?Sized> Drop for Guard<'a, V>
{
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self)
    {
        rsgx_spin_unlock( &mut self.lock );
    }
}


pub struct HashMap<K,V>{
    map:       Box<[Bucket<K,V>]>,
    count:     AtomicUsize,
    rand:      u64,
    bits:      u32,
}


impl<K,V> HashMap<K,V> {
    pub fn new(cap: usize) -> HashMap<K,V> {
        let mut vec = Vec::with_capacity( cap as usize );
        
        let mut rand = 0u64;
        match rsgx_read_rand( as_u8_slice_mut( &mut rand ) ) {
            Ok(_) => (),
            Err(why) => panic!("rsgx_read_rand: {:?}", why),
        };
        
        println!("HashMap::new(cap:{}) => {}ko, size_of::<Bucket>()={}, rand={:x}",
                 cap, cap*mem::size_of_val(&vec)/1024,
                 mem::size_of::<Bucket<K,V>>(),
                 rand );
        
        unsafe { vec.set_len( cap as usize ) };
        let bits = (cap as u64).leading_zeros();

        let mut hashmap = HashMap { map: vec.into_boxed_slice(),
                                    count: ATOMIC_USIZE_INIT,
                                    // collision: ATOMIC_USIZE_INIT,
                                    bits: bits,
                                    rand: rand };
        hashmap.clear();
        hashmap
    }

    pub fn len(&self) -> usize {
        self.count.load( Ordering::Relaxed )
    }

    pub fn capacity(&self) -> usize {
        self.map.len()
    }

    pub fn clear(&mut self) {
        self.count.store( 0, Ordering::Relaxed );
        for b in self.map.iter_mut() {
            b.clear();
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a,K,V> {
        let next = self.map.iter().position(|ref b| b.is_used() );
        Iter { hashmap: self, next: next }
    }
}


pub struct Iter<'a, K:'a ,V: 'a> {
    hashmap: &'a HashMap<K,V>,
    next: Option<usize>,
}


impl<'a, K: 'a, V: 'a> Iterator for Iter<'a,K,V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        match self.next {
            None      => None,
            Some(pos) => {
                self.next = 
                    match self.hashmap.map[pos+1..].iter().position(|ref b| b.is_used() ) {
                        None => None,
                        Some(i) => Some(pos+1+i),
                    };
                Some( (&self.hashmap.map[pos].key, &self.hashmap.map[pos].value) )
            },
        }
    }
}


impl<K: Hash+Copy+PartialEq+Display, V: Default> HashMap<K,V> {

    fn hash(&self, key: K) -> u64 {
        (key.hash() ^ self.rand) >> (64-self.bits)
    }
    
    pub fn insert(&mut self, key: K) -> Guard<V> {
        let mut i = self.hash( key ) as usize;
        let cap = self.capacity();
        let map = self.map.as_mut_ptr();
        let mut nb = 0;
        
        loop {
            i %= cap-1;
            let b = unsafe { &mut *map.offset(i as isize) };
            
            // Check if the slot is free.
            if b.tag.load(Ordering::Relaxed) == 0  {
                // try to use it, if not continue.
                if b.tag.compare_and_swap(0, 1, Ordering::Acquire) == 0 {
                    b.key   = key;
                    b.value = V::default();
                    b.lock = SGX_SPINLOCK_INITIALIZER;
                    
                    // Mark that the slot is correctly initialised.
                    b.tag.store( 2, Ordering::Release );

                    self.count.fetch_add( 1, Ordering::Relaxed );

                    return b.lock()
                }
            }
            
            // Wait until the key is initialized.
            while b.tag.load(Ordering::Relaxed) != 2 {}
            if key == b.key {
                return b.lock()
            }
            
            i += 1;
            nb += 1;
            if nb == cap {
                panic!("HashMap full {} for {}", self.len(), key );
            }
        };
        // unreachable!()
    }
    
    // pub fn get(&self, key: K) -> Option<&V> {
    //     let mut i = self.hash( key ) as usize;
    //     let len = self.len();
        
    //     loop {
    //         i &= len-1;
    //         let b = unsafe { self.table.get_unchecked(i) };
    //         if !b.used {
    //             return None;
    //         } else if key == b.key {
    //             return Some(&b.value)
    //         }
    //         i += 1;
    //     }
    // }

    // pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
    //     let mut i = self.hash( key ) as usize;
    //     let len = self.len();
    //     let table = self.table.as_mut_ptr();
        
    //     loop {
    //         i &= len-1;
    //         let b = unsafe { &mut *table.offset(i as isize) };
    //         if !b.used {
    //             return None;
    //         } else if key == b.key {
    //             return Some(&mut b.value)
    //         }
    //         i += 1;
    //     }
    // }
}
