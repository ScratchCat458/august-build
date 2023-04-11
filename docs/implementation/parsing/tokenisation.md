# Tokenisation

Tokenisation classifies the characters of a string into different categories.
This provides symantic meaning during later parsing steps.

## Side Tangent: Enumeration
In order to assist with error handling, the characters are assigned numbers based on their position.
The tokens contain this information as a span.
These then can be passed to `ariadne` to generate error messages.

## Ident(ifier)
Identifiers are a string of characters, seperated from other tokens by whitespace.
```august
Ident
task_name
named-task
identsdonthavealimitonlengthbythewaybutmaybedontmakeyourfunctionnamestoolong
```

## Punct(uation)
Single character tokens, often used for show relation between tokens.
```august
,
:
;
.
=
+
?
#
@
```

## Literals
Represents a unit of content of either type String, unsigned integer or boolean
### String Literal
Must be surrounded by `"` so that `'` can be used within.
```august
"Hello, World!"
```
### Integer Literal
```august
1234
```
### Boolean Literal
```august
true
false
True
False
```

## Node
A node contains multiple tokens and an encapsulator.
This is a part of the recursive descent model that August uses when parsing.
The encapsulators are:
```august
( )
{ }
[ ]
< >
```
