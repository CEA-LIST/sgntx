DIR_INP_BASE="../data/individualVcf"
DIR_OUT_BASE="../data/comp_enc"

# CASE_DIR="case_small"
# CONTROL_DIR="control_small"
CASE_DIR="case"
CONTROL_DIR="control"

#generate secret key
mkdir -p $DIR_OUT_BASE
# head -c 16 /dev/urandom > $DIR_OUT_BASE/sk.dat
# SK=`cat $DIR_OUT_BASE/sk.dat | xxd -ps`
SK="4c86aaf6afc95e87a68518df8ae75829"

echo "Secret key:" $SK


#encrypt case and control vcfs
function comp_enc_dir {
  DIR_INP=$1
  DIR_OUT=$2

  mkdir -p $DIR_OUT
  parallel -j 1 --bar ./ce/ce $DIR_INP/{/} $DIR_OUT/{/}.ce  $SK ::: $DIR_INP/*.vcf 
}

comp_enc_dir $DIR_INP_BASE/$CASE_DIR $DIR_OUT_BASE/$CASE_DIR
comp_enc_dir $DIR_INP_BASE/$CONTROL_DIR $DIR_OUT_BASE/$CONTROL_DIR

