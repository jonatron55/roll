use anstream::eprintln;
use anstyle::{AnsiColor, Color, Style};

const HEADER: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::BrightGreen)))
    .bold();
const CMD: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::BrightCyan)))
    .bold();
const ARG: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::BrightMagenta)))
    .italic();
const EM: Style = Style::new().italic();

pub fn print_help() {
    let cmd = env!("CARGO_PKG_NAME");
    eprintln!(
        "
This command evaluates a dice expression and prints the result.

{HEADER}Usage:{HEADER:#} {CMD}{cmd}{CMD:#} {ARG}[mode] <expression>{ARG:#}

{HEADER}Modes:{HEADER:#}
  If no mode is specified, the expression will be evaluated with randomly rolled
  dice. Otherwise, the following modes may appear as the first argument:

    {CMD}min{CMD:#}: evaluate the expression as if all dice rolls were 1.
    {CMD}mid{CMD:#}: evaluate the expression as if all dice rolls were their median value,
         which is not necessarily the same as the median of the total
         distribution of the expression. This will always be a fraction ending
         in 0.5 for even-sided dice, but the final result will still be rounded
         down to the nearest integer.
    {CMD}max{CMD:#}: evaluate the expression as if all dice rolls were the highest possible.
    {CMD}dot{CMD:#}: output the expression's syntax tree in Graphviz DOT format.
    {CMD}mermaid{CMD:#}: output the expression's syntax tree in Mermaid format.

{HEADER}Expressions:{HEADER:#}
  Dice expressions use typical notation such as {CMD}3d8 + 2{CMD:#}. The normal arithmetic
  operations {CMD}+{CMD:#}, {CMD}-{CMD:#}, {CMD}*{CMD:#}, and {CMD}/{CMD:#} are supported with {CMD}×{CMD:#} and {CMD}÷{CMD:#} recognized as alternate
  forms of {CMD}*{CMD:#} and {CMD}/{CMD:#}. Products precede sums unless grouped by parentheses. Dice
  rolls are expressed as {ARG}count{ARG:#}{CMD}d{CMD:#}{ARG}sides{ARG:#} where {ARG}sides{ARG:#} is {CMD}4{CMD:#}, {CMD}6{CMD:#}, {CMD}8{CMD:#}, {CMD}10{CMD:#}, {CMD}12{CMD:#}, {CMD}20{CMD:#}, or {CMD}100{CMD:#}
  (the sequence {CMD}d%{CMD:#} is interpreted as {CMD}d100{CMD:#}). If the die count is omitted (e.g.
  {CMD}d20 + 5{CMD:#}), it is assumed to be 1, and if the number of sides is omitted (e.g.
  {CMD}4d + 1{CMD:#}), then the dice are assumed to be six-sided. A roll may be followed by
  any number of selection modifiers, to keep or discard certain dice:

    • {CMD}k{CMD:#}{ARG}n{ARG:#} or {CMD}kh{CMD:#}{ARG}n{ARG:#}: keep the highest {ARG}n{ARG:#} dice. If {ARG}n{ARG:#} is omitted, it is assumed to be
      {CMD}1{CMD:#}.
    • {CMD}kl{CMD:#}{ARG}n{ARG:#}: keep the lowest {ARG}n{ARG:#} dice. If {ARG}n{ARG:#} is omitted, it is assumed to be {CMD}1{CMD:#}.
    • {CMD}d{CMD:#}{ARG}n{ARG:#} or {CMD}dl{CMD:#}{ARG}n{ARG:#}: discard the lowest {ARG}n{ARG:#} dice. If {ARG}n{ARG:#} is omitted, it is assumed to be
      {CMD}1{CMD:#}.
    • {CMD}dh{CMD:#}{ARG}n{ARG:#}: discard the highest {ARG}n{ARG:#} dice. If {ARG}n{ARG:#} is omitted, it is assumed to be {CMD}1{CMD:#}.
    • {CMD}adv{CMD:#} or {CMD}ad{CMD:#}: reroll the preceding expression and take the higher result.
    • {CMD}dis{CMD:#} or {CMD}da{CMD:#}: reroll the preceding expression and take the lower result.

  For example, to roll 4d6 and keep the highest 3 (common in D&D character
  generation), you could write {CMD}4d6kh3{CMD:#} or equivalently {CMD}4d6d1{CMD:#} (roll 4d6 and
  discard the lowest 1). Though uncommon, it is possible to chain several
  selections and they will be evaluated in order from left to right. {CMD}adv{CMD:#} and
  {CMD}dis{CMD:#} refer to \"advantage\" and \"disadvantage\" respectively, common in D&D
  5e and they have the effect of rerolling the entire previous sub-expression
  (including any previous selections) and taking the higher or lower total
  respectively.

  Only integers are supported as input, and the result of an expression is
  always an integer. However, intermediate values may be non-integer, for
  example when using the {CMD}mid{CMD:#} option or as the result of division. The final
  result is always rounded {EM}down{EM:#} to the nearest integer (positive numbers towards
  zero and negative numbers away from zero).

{HEADER}Formal grammar:{HEADER:#}
  Expressions are defined by the following EBNF grammar:

  {CMD}root{CMD:#} = {CMD}sum{CMD:#};
  {CMD}sum{CMD:#} = {CMD}term{CMD:#}, {{ ({ARG}\"+\"{ARG:#} | {ARG}\"-\"{ARG:#}), {CMD}term{CMD:#} }};
  {CMD}term{CMD:#} = {CMD}factor{CMD:#}, {{ ({ARG}\"*\"{ARG:#} | {ARG}\"×\"{ARG:#} | {ARG}\"/\"{ARG:#} | {ARG}\"÷\"{ARG:#}), {CMD}factor{CMD:#} }};
  {CMD}factor{CMD:#} = {ARG}\"(\"{ARG:#}, {CMD}sum{CMD:#}, {ARG}\")\"{ARG:#} | {CMD}negation{CMD:#} | {CMD}integer{CMD:#} | {CMD}roll{CMD:#};
  {CMD}negation{CMD:#} = {ARG}\"-\"{ARG:#}, {CMD}factor{CMD:#};
  {CMD}roll{CMD:#} = [{CMD}integer{CMD:#}], {ARG}\"d\"{ARG:#}, [{CMD}integer{CMD:#}], [{CMD}selection{CMD:#}];
  {CMD}selection{CMD:#} = (
      {ARG}\"k\"{ARG:#}, {CMD}integer{CMD:#} |
      {ARG}\"kh\"{ARG:#}, {CMD}integer{CMD:#} |
      {ARG}\"kl\"{ARG:#}, {CMD}integer{CMD:#} |
      {ARG}\"d\"{ARG:#}, {CMD}integer{CMD:#} |
      {ARG}\"dh\"{ARG:#}, {CMD}integer{CMD:#} |
      {ARG}\"dl\"{ARG:#}, {CMD}integer{CMD:#} |
      {ARG}\"adv\"{ARG:#} | {ARG}\"ad\"{ARG:#} |
      {ARG}\"dis\"{ARG:#} | {ARG}\"da\"{ARG:#}
    ), [{CMD}selection{CMD:#}];
  {CMD}integer{CMD:#} = {CMD}digit{CMD:#}, {{{CMD}digit{CMD:#}}};
  {CMD}digit{CMD:#} = {ARG}\"0\"{ARG:#} | {ARG}\"1\"{ARG:#} | {ARG}\"2\"{ARG:#} | {ARG}\"3\"{ARG:#} | {ARG}\"4\"{ARG:#} | {ARG}\"5\"{ARG:#} | {ARG}\"6\"{ARG:#} | {ARG}\"7\"{ARG:#} | {ARG}\"8\"{ARG:#} | {ARG}\"9\"{ARG:#};
"
    );
}
