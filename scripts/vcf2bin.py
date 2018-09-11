CONST_LEN_LINE=True #compressed line has 16 bytes, otherwise the ID field is written as null terminated string

def vcf_file_to_bin(inp_fname, out_fname):
  import sys
  f = open(inp_fname,'r')
  fo = open(out_fname, 'wb')
  for line in f.readlines():
    line = line.strip()
    if not line.startswith('#'): 
      line_split = line.split('\t')
      chrom = int(line_split[0])
      pos = int(line_split[1])
      if CONST_LEN_LINE:
        iden = int(line_split[2][2:])
      else:
        iden = line_split[2]
      ref = line_split[3][:1]
      alt = line_split[4][:1]
      is_hetero = line_split[-1] == 'heterozygous'

      #compress each line to 16 bytes
      buff = bytes()
      buff += chrom.to_bytes(1,sys.byteorder)
      buff += pos.to_bytes(4,sys.byteorder)
      if CONST_LEN_LINE:
        buff += iden.to_bytes(8,sys.byteorder)
      else:
        buff += bytes(iden+'\0', 'utf8') 
      buff += bytes(ref, 'utf8')
      buff += bytes(alt, 'utf8')
      buff += is_hetero.to_bytes(1,sys.byteorder)
      fo.write(buff) 

  f.close()

if __name__ == '__main__':
  import os
  from joblib import Parallel, delayed
  from argparse import ArgumentParser
  
  print('Start')
  parser = ArgumentParser()
  parser.add_argument('-i', '--inp_path', required=True)
  parser.add_argument('-o', '--out_path', required=True)
  args = parser.parse_args()
  inp_path = args.inp_path + '/'
  out_path = args.out_path + '/'

  os.makedirs(out_path, exist_ok=True)

  Parallel(n_jobs=-1, verbose=5)(delayed(vcf_file_to_bin)(inp_path + fname, out_path + fname + '.bin') for fname in os.listdir(inp_path))
      
