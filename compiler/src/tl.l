%%

[0-9]+          "DEC_INT"
0x[0-9a-fA-F]+  "HEX_INT"
0b[01]+         "BIN_INT"

\|\|              "LOR"
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

[a-zA-Z_][a-zA-Z0-9_]*  "IDENT"

"(?:[^"\\]|\\.)*"       "STRING"
'(?:[^"\\]|\\.)'        "CHAR"

(?:(?://|#)[^\n]*\n)     ;
[\t\r\n ]+ ;