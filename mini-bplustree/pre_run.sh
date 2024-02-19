./compile.sh

if [ -f sim_disk ]; then
    rm sim_disk
fi

dd if=/dev/zero of=sim_disk bs=1G count=8