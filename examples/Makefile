hello_world: hello_world.o
	ld65 -o hello_world hello_world.o -C bios.cfg

hello_world.o: hello_world.s
	ca65 hello_world.s
