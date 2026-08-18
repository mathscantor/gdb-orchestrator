set confirm off
set pag off
set print pretty on
set output-radix 16
set disassembly-flavor intel
set print elements 0
set cwd .

record btrace
set record function-call-history-size unlimited

b func_1
commands 1
  record function-call-history /ilc 1
  continue
end
continue

