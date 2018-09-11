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



use core::cell::UnsafeCell;
use core::ops::{Drop, Deref, DerefMut};
use sgx_types::{sgx_spinlock_t, sgx_spin_lock, sgx_spin_unlock, SGX_SPINLOCK_INITIALIZER};



fn rsgx_spin_lock(lock: &mut sgx_spinlock_t) {
    unsafe { sgx_spin_lock( lock ); }
}

fn rsgx_spin_unlock(lock: &mut sgx_spinlock_t) {
    unsafe { sgx_spin_unlock( lock ); }
}



pub struct Mutex<T: ?Sized> {
    lock: sgx_spinlock_t,
    data: UnsafeCell<T>,
}


pub struct MutexGuard<'a, T: ?Sized + 'a>
{
    lock: &'a mut sgx_spinlock_t,
    data: &'a mut T,
}


unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}


impl<T> Mutex<T>
{
    pub fn new(user_data: T) -> Mutex<T>
    {
        Mutex
        {
            lock: SGX_SPINLOCK_INITIALIZER,
            data: UnsafeCell::new(user_data),
        }
    }
}


impl<T: ?Sized> Mutex<T>
{
    pub fn lock(&mut self) -> MutexGuard<T>
    {
        rsgx_spin_lock( &mut self.lock );

        MutexGuard
        {
            lock: &mut self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }
}   


impl<'a, T: ?Sized> Deref for MutexGuard<'a, T>
{
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.data }
}


impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T>
{
    fn deref_mut<'b>(&'b mut self) -> &'b mut T { &mut *self.data }
}


impl<'a, T: ?Sized> Drop for MutexGuard<'a, T>
{
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self)
    {
        rsgx_spin_unlock( &mut self.lock );
    }
}
