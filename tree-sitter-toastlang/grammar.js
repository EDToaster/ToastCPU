/**
 * Tree-sitter grammar for ToastLang (.tl).
 *
 * Hand-translated from the compiler's real grammar:
 *   - tokens:  compiler/src/tl.l   (grmtools lexer)
 *   - rules:   compiler/src/tl.y   (grmtools/yacc grammar)
 *
 * This grammar exists only to drive *syntax highlighting* in tree-sitter
 * editors (e.g. Helix), which do not consume the language server's semantic
 * tokens. Correctness-critical parsing lives in tl.y and is reused by both the
 * compiler and the language server; keep this file in sync with tl.l/tl.y when
 * the language changes.
 *
 * Where tl.y uses a single `Identifier` token for every role, this grammar
 * deliberately diverges so highlight queries can tell them apart:
 *   - identifiers in type position are aliased to `type_identifier`
 *   - bare word / call statements are aliased to `word`
 *   - declaration names are kept as `identifier` under named fields
 */

module.exports = grammar({
  name: 'toastlang',

  word: $ => $.identifier,

  extras: $ => [/\s/, $.comment],

  rules: {
    source_file: $ => repeat($._item),

    _item: $ => choice(
      $.function_definition,
      $.global,
      $.inline,
      $.struct_definition,
      $.module,
      $.using,
    ),

    // mod name { ... }
    module: $ => seq(
      'mod',
      field('name', $.identifier),
      '{',
      repeat($._item),
      '}',
    ),

    // fn name <inputs> -> <outputs> { body }
    function_definition: $ => seq(
      'fn',
      field('name', $.identifier),
      $.func_type,
      field('body', $.block),
    ),

    // global name [n]? type init
    global: $ => seq(
      'global',
      field('name', $.identifier),
      optional($.array_size),
      field('type', $._type),
      field('value', $._int_literal),
    ),

    // inline name <statement>
    inline: $ => seq(
      'inline',
      field('name', $.identifier),
      $._statement,
    ),

    // using name
    using: $ => seq('using', field('name', $.identifier)),

    // struct name { members }
    struct_definition: $ => seq(
      'struct',
      field('name', $.identifier),
      '{',
      repeat($.struct_member),
      '}',
    ),

    // member_name [n]? type
    struct_member: $ => seq(
      field('name', $.identifier),
      optional($.array_size),
      field('type', $._type),
    ),

    array_size: $ => seq('[', $._int_literal, ']'),

    block: $ => seq('{', repeat($._statement), '}'),

    _statement: $ => choice(
      $.number,
      $.char,
      $.string,
      $.boolean,
      alias($.identifier, $.word), // a bare word: a function/call reference
      $._operator,
      $.block,
      $.unroll,
      $.if,
      $.while,
      $.let,
    ),

    unroll: $ => seq('unroll', $._int_literal, $.block),

    if: $ => seq('if', $.block, optional(seq('else', $.block))),

    while: $ => seq('while', field('condition', $.block), field('body', $.block)),

    // let a b { ... } -- binds named locals
    let: $ => seq('let', repeat(field('binding', $.identifier)), $.block),

    // Operators (see the Operator rule in tl.y).
    _operator: $ => choice(
      '||', '&&', '!',
      '++', '--',
      '+', '-',
      '|', '&', '~',
      '>>>', '>>', '<<', '^',
      '=', '<=', '>=', '<', '>',
      '??',
      'return',
      $.cast,
      $.size_of,
      $.ptr_expression,
      $.call_operator,
      $.struct_access,
      $.const_array_access,
    ),

    cast: $ => seq('as', '(', field('type', $._type), ')'),
    size_of: $ => seq('sizeof', '(', field('type', $._type), ')'),
    ptr_expression: $ => seq('ptr', '(', field('target', $.identifier), ')'),
    call_operator: $ => seq('(', ')'),
    const_array_access: $ => seq('[', $._int_literal, ']'),

    // .field  (field access); the field name may be module-qualified.
    struct_access: $ => token(seq('.', /[a-zA-Z_][a-zA-Z0-9_:]*/)),

    // Types (Type / BaseType / GenType / PtrType / FuncType in tl.y).
    func_type: $ => seq(
      field('inputs', repeat($._type)),
      '->',
      field('outputs', repeat($._type)),
    ),

    _type: $ => choice(
      $.pointer_type,
      $._type_atom,
    ),

    _type_atom: $ => choice(
      alias($.identifier, $.type_identifier),
      $.generic_type,
      seq('(', $.func_type, ')'),
    ),

    generic_type: $ => seq('$', alias($.identifier, $.type_identifier)),

    pointer_type: $ => prec.left(seq($._type, '*')),

    _int_literal: $ => choice($.number, $.char),

    // Literals (see tl.l).
    number: $ => token(/-?(0x[0-9a-fA-F_]+|0b[01_]+|[0-9][0-9_]*)/),
    char: $ => token(/'([^'\\]|\\.)'/),
    string: $ => token(/"([^"\\]|\\.)*"/),
    boolean: $ => choice('true', 'false'),

    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_:]*/,

    // // line comment   or   # line comment (incl. #include<...> directives)
    comment: $ => token(choice(
      seq('//', /[^\n]*/),
      seq('#', /[^\n]*/),
    )),
  },
});
