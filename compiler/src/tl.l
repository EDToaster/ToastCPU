%%

true            "TRUE_BOOL"
false           "FALSE_BOOL"

[0-9]+          "DEC_INT"
0x[0-9a-fA-F]+  "HEX_INT"
0b[01]+         "BIN_INT"

\?\?            "HOLE"

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

\[              "LS"
\]              "RS"

\{              "LB"
\}              "RB"

\$               "GENERIC"
\*               "PTR"

global          "GLOBAL"
inline          "INLINE"
fn              "FN"
let             "LET"
struct          "STRUCT"
mod             "MOD"
using           "USING"
->              "RARROW"

unroll          "UNROLL"
if              "IF"
else            "ELSE"
while           "WHILE"
ptr             "PTR_OP"
as              "AS"
sizeof          "SIZEOF"
return          "RETURN"

[a-zA-Z_][a-zA-Z0-9_:]*          "IDENT"
\.[a-zA-Z_][a-zA-Z0-9_:]*        "STRUCT_ACCESS"

"(?:[^"\\]|\\.)*"               "STRING"
'(?:[^"\\]|\\.)'                "CHAR"

(?:(?://|#)[^\n]*\n)            ;
[\t\r\n ]+                      ;

.               "UNMATCHED"