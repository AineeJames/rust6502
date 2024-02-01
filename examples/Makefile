# Makefile

ASSEMBLER = ca65
LINKER = ld65
ASSEMBLY_FILES = $(wildcard *.s)
OBJECT_FILES = $(ASSEMBLY_FILES:.s=.o)

all: $(OBJECT_FILES)

%.o: %.s
	$(ASSEMBLER) $<

	$(LINKER) -o $(basename $<) -C bios.cfg $@

clean:
	rm -f $(OBJECT_FILES)

SUBDIRS := $(wildcard */.)

all: $(SUBDIRS)
$(SUBDIRS):
		$(MAKE) -C $@

.PHONY: all $(SUBDIRS)
