%%

[0-9]+          "DEC_INT"
0x[0-9a-fA-F]+  "HEX_INT"
0b[01]+         "BIN_INT"

\?\?              "HOLE"

\|\|            "LOR"
&&              "LAND"
!               "LNOT"

\+              "ADD"
-               "SUB"
\|               "BOR"
&               "BAND"
~               "BNOT"
>>              "SSHR"
>>>             "SHR"
<<              "SHL"
\^              "XOR"

=               "EQ"
<               "LT"
<=              "LTE"
>               "GT"
>=              "GTE"

\(              "LP"
\)              "RP"

\{              "LB"
\}              "RB"

global          "GLOBAL"
fn              "FN"
let             "LET"
struct          "STRUCT"
->              "RARROW"

unroll          "UNROLL"
if              "IF"
else            "ELSE"
while           "WHILE"
as              "AS"

\$?[a-zA-Z_][a-zA-Z0-9_]*\**  "IDENT"

"(?:[^"\\]|\\.)*"       "STRING"
'(?:[^"\\]|\\.)'        "CHAR"

(?:(?://|#)[^\n]*\n)     ;
[\t\r\n ]+ ;