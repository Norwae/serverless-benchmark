#!/bin/bash

DATADIR=$(uuidgen)

mkdir $DATADIR

for ((i = 0; $i < 50; i++)) 
do
	ID=$(uuidgen)
	NAMES[$i]=$ID
	dd if=/dev/urandom of=$DATADIR/$ID count=1 bs=10M 2>/dev/null
done

for ((i = 0; $i < 1000; i++))
do
	echo $i / 1000
	INDEX=$(($RANDOM % 50)) 
	FILE=$DATADIR/${NAMES[$INDEX]}
	aws s3 cp $FILE s3://$PERFCOMRUST/$(uuidgen) >/dev/null
	aws s3 cp $FILE s3://$PERFCOMJAVA/$(uuidgen) >/dev/null
done

rm -r $DATADIR
