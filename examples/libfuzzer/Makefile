RM = del
CP = copy
CC = cl.exe
LD = link.exe
CFLAGS = /nologo /MD /fsanitize=address /Zi /fsanitize-coverage=inline-8bit-counters /fsanitize-coverage=edge /fsanitize-coverage=trace-cmp /fsanitize-coverage=trace-div
LDFLAGS = /nologo /debug /SUBSYSTEM:CONSOLE /INFERASANLIBS

all: libfuzzer_harness.exe

.PHONY: rusty_radamsa.dll.lib
rusty_radamsa.dll.lib:
	cd ..\..\ && cargo build
	$(CP) ..\..\target\debug\$@ .\$@

libfuzzer_harness.cc.obj: libfuzzer_harness.cc
	$(CP) ..\..\rusty_radamsa.h .\rusty_radamsa.h
	$(CC) /c /Fo$@ $(CFLAGS) $**

libfuzzer_harness.exe: rusty_radamsa.dll.lib libfuzzer_harness.cc.obj
	$(CP) ..\..\target\debug\rusty_radamsa.dll .\rusty_radamsa.dll 
	$(LD) $(LDFLAGS) /out:$@ $** clang_rt.fuzzer_MD-x86_64.lib libsancov.lib /WHOLEARCHIVE:rusty_radamsa.dll.lib Advapi32.lib Userenv.lib Ws2_32.lib Bcrypt.lib

.PHONY: clean
clean:
	$(RM) *.obj
	$(RM) libfuzzer_harness.exe
	$(RM) libfuzzer_harness.lib
	$(RM) rusty_radamsa.dll.lib
	$(RM) rusty_radamsa.dll
	$(RM) rusty_radamsa.h
	$(RM) crash-*
	$(RM) *.exp
	$(RM) *.pdb