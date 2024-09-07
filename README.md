Dice Roller
===========

This is a simple Rust program to parse and evaluate dice expressions using typical notation, for example `3d8 + 2`. The normal arithmetic  operations `+`, `-`, `*`, and `/` are supported. `×` and `÷` are recognized as well as `*` and `/`. Products precede sums unless grouped be parantheses. Dice rolls are expressed as &lt;*number of dice*&gt;`d`&lt;*number of sides*&gt; where &lt;*number of sides*&gt; is 4, 6, 8, 10, 12, 20, or 100 (the sequence `d%` is interpreted as `d100`). If the die count is omitted (e.g. `d20 + 5`), it is assumed to be 1 and if the number of sides are omitted (e.g. `4d + 1`), then the dice are assumed to be six-sided. A roll by be followed by any number of selection modifiers, too keep or discard certain dice:

 - `k<n>` or `kh<n>`: keep the highest `<n>` dice.
 - `kl<n>`: keep the lowest `<n>` dice.
 - `d<n>` or `dl<n>`: discard the lowest `<n>` dice.
 - `dh<n>`: discard the highest `<n>` dice.
 - `adv` or `ad`: reroll the preceding expression and take the higher result.
 - `dis` or `da`: reroll the preceding expression and take the lower result.

If `<n>` is omitted, it is taken to be 1. For example, to roll 4d6 and keep the highest 3 (common in D&D character generation), you could write `4d6kh3` or equivalently `4d6d1` (roll 4d6 and discard the lowest). Though uncommon, it is possible to chain several selections and they will be evaluated in order from left to right. `adv` and `dis` refer to "advantage" and "disadvantage" respectively, common in D&D 5e and they have the effect of rolling the entire previous expression (including any previous selections) twice and taking the higher or lower total respectively.

Only integers are supported, and the result of an expression is always an integer. When division is performed, the result is rounded down to the nearest integer before the next operation is performed.

Usage
-----

The program reads a single expression from the command line and prints both the total result and the individual dice rolls. For example:

```
> roll 4d6d1
[d6:6] [d6:4] [d6:4] [d6:1]
total = 14
```

Grammar
-------

Dice expressions are parsed according to the following grammar:

```text
root -> sum
sum -> term [('+' | '-') term]
term -> product | sum
product -> factor [('*' | '/') factor]
factor -> '(' sum ')' | negation | integer | roll | product
negation -> '-' factor
roll -> [integer] 'd' [integer] [selection]
selection -> (
        'k' integer |
        'kh' integer |
        'kl' integer |
        'd' integer |
        'dh' integer |
        'dl' integer |
        'adv' | 'ad' |
        'dis' | 'da'
    ) [selection]
integer -> /[0-9]+/
```
