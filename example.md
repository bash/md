+++
title = "Markdown"
description = "Here goes a short description or subtitle for this page"
draft = true
+++

# Markdown `Showcase`

## Formatting Examples
**Lorem** _ipsum_ [dolor sit](https://example.com) ~~amet~~, `consectetur`[^1] adipiscing elit, sed do eiusmod tempor incididunt[^2] ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

[^1]: Here comes the footnote

## Nested Formatting
Here comes *italic with nested ~~strikethrough~~, then **bold**, and finally `some code`*.

---

## H2

Ac turpis egestas integer eget aliquet nibh praesent tristique magna. Amet luctus venenatis lectus magna fringilla urna porttitor. Ultricies mi quis hendrerit dolor. Risus in hendrerit gravida rutrum quisque non. Eleifend mi in nulla posuere sollicitudin aliquam.

### H3
Duis ut diam quam nulla porttitor massa id neque. Pharetra convallis posuere morbi leo urna molestie. Nunc mattis enim ut tellus elementum. In iaculis nunc sed augue lacus viverra vitae congue eu. Dolor sit amet consectetur adipiscing elit pellentesque habitant morbi. Lacinia at quis risus sed vulputate odio. Commodo odio aenean sed adipiscing diam donec. Morbi tristique senectus et netus et. Ut eu sem integer vitae justo eget.

## Code
```python
print('Hello World')
```

### Empty Block
```js
```

## Math
### Display

$$\left( \sum_{k=1}^n a_k b_k \right)^2 \leq \left( \sum_{k=1}^n a_k^2 \right) \left( \sum_{k=1}^n b_k^2 \right)$$

### Inline
Some inline math: $e = mc^2$

## Quote
> Don't believe everything you read on the internet.
- Nikola Tesla

### Empty
>

## Admonitions
> [!TIP]
> For more examples see [example.md](./example.md).

## Block Quote Followed by List
> Block quote
- Foo
- Bar
- Baz

## Ordered List
1. One
2. Two
   1. Nested
   2. Things
3. Three

## Unordered List
* Foo
* Bar
* Baz

## List Formatting
1. **Bold Text**
2. Code Block
   ```js
   console.log('hello world')
   ```
   with text after code block
3. Sublist with text before
   * foo
   * bar
   * baz

## Quoted List
> 1. Wow
> 2. What
> 3. A
> 4. Quote

## Deeply Nested List
1. Wow
   1. Much
      1. Nesting
         1. Such
            1. Deep
               1. List
* Wow
   * Much
      * Nesting
         * Such
            * Deep
               * List


## Tasks
* [ ] buy groceries
* [x] clean kitchen
* [x] fix bathroom lights

## Details

<details>
<summary>Expand for more :)</summary>

Amet luctus venenatis lectus magna fringilla urna porttitor. Ultricies mi quis hendrerit dolor. 

</details>

## Table

| Terminal            | Iterations | min          | max           | mean         |
|---------------------|------------|--------------|---------------|--------------|
| foot                | 10000      | 26.130 µs    | 248.260 µs    | 31.825 µs    |
| XTerm               | 10000      | 33.550 µs    | 295.990 µs    | 39.926 µs    |
| Konsole             | 10000      | 34.110 µs    | 3.652145 ms   | 38.094 µs    |
| Alacritty           | 10000      | 40.340 µs    | 414.961 µs    | 57.569 µs    |
| IntelliJ IDEA       | 10000      | 71.267 µs    | 2.453094 ms   | 154.491 µs   |
| Terminal.app        | 10000      | 196.143 µs   | 25.064408 ms  | 241.916 µs   |
| Hyper               | 10000      | 16.287473 ms | 57.534790 ms  | 20.040066 ms |
| GNOME Console (vte) | 10000      | 8.157828 ms  | 56.823240 ms  | 20.656316 ms |
| VSCode              | 10000      | 24.164008 ms | 140.036258 ms | 26.061349 ms |
| iTerm2              | 10000      | 4.065856 ms  | 49.872777 ms  | 28.259948 ms |


## Title with `code` and *emph*

# H1
## H2
### H3
Notice that there is no extra space between these nested headings.


[^2]: Aaaand a second footnote
