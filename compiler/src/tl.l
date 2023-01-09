%%
[0-9]+          "DEC_INT"
0x[0-9a-fA-F]+  "HEX_INT"
0b[01]+         "BIN_INT"

\+              "ADD"
-               "SUB"

\{               "LB"
\}               "RB"

fn              "FN"
->              "RARROW"

[a-zA-Z_][a-zA-Z0-9_]*  "IDENT"

[\t\r\n ]+ ;