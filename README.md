`find ./disk/ -type f | tar --xform='s/.*\///g' -cf disk.tar --format=ustar --files-from=/dev/stdin`
