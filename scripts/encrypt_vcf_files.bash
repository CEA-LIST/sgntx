DIR_INP_BASE="../data/individualVcf"
DIR_OUT_BASE="../data/encrypted"

#generate secret key
mkdir -p $DIR_OUT_BASE
head -c 16 /dev/urandom > $DIR_OUT_BASE/sk.dat
SK=`cat $DIR_OUT_BASE/sk.dat | xxd -ps`

echo "Secret key:" $SK


#encrypt case and control vcfs
function encrypt_dir {
    DIR_INP=$1
    DIR_OUT=$2
    
    mkdir -p $DIR_OUT
    for FILE in $DIR_INP/*.vcf
    do
	FN=$(basename $FILE)
	echo $FN

	head -c 8 /dev/urandom > $DIR_OUT/$FN
	head -c 8 /dev/zero >> $DIR_OUT/$FN	
	IV=`cat $DIR_OUT/$FN | xxd -ps`

	openssl enc -aes-128-ctr -e -K $SK -iv $IV -in $DIR_INP/$FN -out $DIR_OUT/$FN.enc
	cat $DIR_OUT/$FN.enc >> $DIR_OUT/$FN
	rm $DIR_OUT/$FN.enc
    done
}

encrypt_dir $DIR_INP_BASE/case $DIR_OUT_BASE/case
encrypt_dir $DIR_INP_BASE/control $DIR_OUT_BASE/control

