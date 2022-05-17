all: wrapper

wrapper: wrapper.o
	ar rcs libwrapper.a wrapper.o

wrapper.o:
	gcc -O -c wrapper.c wrapper.h -Wall -I${C_INCLUDE_PATH}

clean:
	rm -f *.o *.a *.gch 