RM = del
CP = copy
CC = cl.exe
LD = link.exe
LDFLAGS = /nologo /debug /SUBSYSTEM:CONSOLE

all: simple.exe

.PHONY: rusty_radamsa.dll.lib
rusty_radamsa.dll.lib:
	cd ..\..\ && cargo build
	$(CP) ..\..\target\debug\$@ .\$@

simple.cc.obj: simple.cc
	$(CP) ..\..\rusty_radamsa.h .\rusty_radamsa.h
	$(CC) /c /Fo$@ $(CFLAGS) $**

simple.exe: rusty_radamsa.dll.lib simple.cc.obj
	$(CP) ..\..\target\debug\rusty_radamsa.dll .\rusty_radamsa.dll 
	$(LD) $(LDFLAGS) /out:$@ $** /WHOLEARCHIVE:rusty_radamsa.dll.lib Advapi32.lib Userenv.lib Ws2_32.lib Bcrypt.lib

.PHONY: clean
clean:
	$(RM) *.obj
	$(RM) simple.exe
	$(RM) rusty_radamsa.h
	$(RM) rusty_radamsa.dll.lib
	$(RM) rusty_radamsa.dll
	$(RM) crash-*
	$(RM) *.exp
	$(RM) *.pdb
	cd .. && cargo clean