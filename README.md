# inari-graph

<p align="center">
  <img src="images/cover.gif"><br>
  The graph of sin(<i>x</i> ± sin <i>y</i>) (sin <i>x</i> ± <i>y</i>) = cos(sin((sin <i>x</i> ± cos <i>y</i>) (sin <i>y</i> ± cos <i>x</i>))) over [4, 6.5] × [2, 4.5].
</p>

inari-graph can plot the graph of a relation like above in a reliable manner. The method is based on algorithms 1.1–3.2, 3.4.1 and 3.4.2 of [Tup01].

## Usage

1. Install Rust

   https://www.rust-lang.org/tools/install

1. Build

   ```bash
   git clone https://github.com/mizuno-gsinet/inari-graph.git
   cd inari-graph
   cargo build --release
   ```

1. Run

   ```bash
   ./target/release/inari-graph graph.png "y == sin(x)"
   ```
   
   Use `-b <xmin> <xmax> <ymin> <ymax>` option to change the bounds. The default is `-10 10 -10 10`.
   
   Use `-s <width> <height>` option to change the size of the output image. The default is `1024 1024`.
   
   Try some [example relations](Examples.md) or your own one.

## Color Legend

- ![Black](images/black.png) There is at least one point that satisfies the relation in the pixel.
- ![Blue](images/blue.png) Uncertain.
- ![White](images/white.png) There is no points that satisfy the relation in the pixel.

## Syntax

### Expression

| Input                                   | Interpreted as                                               | Notes                                                        |
| --------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| `-x`                                    | −*x*                                                         |                                                              |
| `x + y`                                 | *x* + *y*                                                    |                                                              |
| `x - y`                                 | *x* − *y*                                                    |                                                              |
| `x * y`                                 | *x* *y*                                                      |                                                              |
| `x / y`                                 | *x* / *y*                                                    | Undefined for *y* = 0.                                       |
| `sqrt(x)`                               | √*x*                                                         | Undefined for *x* < 0.                                       |
| `x^n`                                   | *x*<sup>*n*</sup>                                            | `n` must be an integer constant.<br />Repetition like `x^2^3` is not supported.<br />See also `exp`, `exp2` and `exp10`. |
| `exp(x)`<br />`exp2(x)`<br />`exp10(x)` | e<sup>*x*</sup><br />2<sup>*x*</sup><br />10<sup>*x*</sup>   |                                                              |
| `log(x)`<br />`log2(x)`<br />`log10(x)` | log<sub>e</sub> *x*<br />log<sub>2</sub> *x*<br />log<sub>10</sub> *x* | Undefined for *x* ≤ 0.                                       |
| `sin(x)`                                | sin *x*                                                      |                                                              |
| `cos(x)`                                | cos *x*                                                      |                                                              |
| `tan(x)`                                | tan *x*                                                      | Undefined for *x* = (*n* + 1/2)π.                            |
| `asin(x)`                               | sin<sup>-1</sup> *x*                                         | Undefined for *x* < −1 and *x* > 1.                          |
| `acox(x)`                               | cos<sup>-1</sup> *x*                                         | Undefined for *x* < −1 and *x* > 1.                          |
| `atan(x)`                               | tan<sup>-1</sup> *x*                                         |                                                              |
| `atan2(y, x)`                           | tan<sup>-1</sup>(*y* / *x*)                                  | Undefined for (*x*, *y*) = (0, 0).                           |
| `sinh(x)`                               | sinh *x*                                                     |                                                              |
| `cosh(x)`                               | cosh *x*                                                     |                                                              |
| `tanh(x)`                               | tanh *x*                                                     |                                                              |
| `asinh(x)`                              | sinh<sup>-1</sup> *x*                                        |                                                              |
| `acosh(x)`                              | cosh<sup>-1</sup> *x*                                        | Undefined for *x* < 1.                                       |
| `atanh(x)`                              | tanh<sup>-1</sup> *x*                                        | Undefined for *x* ≤ −1 and *x* ≥ 1.                          |
| `abs(x)`                                | \|*x*\|                                                      |                                                              |
| `min(x, y)`                             | min {*x*, *y*}                                               |                                                              |
| `max(x, y)`                             | max {*x*, *y*}                                               |                                                              |
| `floor(x)`                              | ⌊*x*⌋                                                        | [The floor function.](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions) |
| `ceil(x)`                               | ⌈*x*⌉                                                        | [The ceiling function.](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions) |
| `sgn(x)`                                | sgn(*x*)                                                     | [The sign function.](https://en.wikipedia.org/wiki/Sign_function) |
| `mod(x, y)`                             | *x* mod *y*                                                  | [The modulo operation.](https://en.wikipedia.org/wiki/Modulo_operation)<br />The value is nonnegative, *i.e.*, 0 ≤ *x* mod *y* < *y*. |

### Relation

| Input    | Interpreted as | Notes                                                        |
| -------- | -------------- | ------------------------------------------------------------ |
| `x == y` | *x* = *y*      |                                                              |
| `x < y`  | *x* < *y*      |                                                              |
| `x <= y` | *x* ≤ *y*      |                                                              |
| `x > y`  | *x* > *y*      |                                                              |
| `x >= y` | *x* ≥ *y*      |                                                              |
| `X && Y` | *X* ∧ *Y*      | [Logical conjunction.](https://en.wikipedia.org/wiki/Logical_conjunction)<br />`X` and `Y` must be a relation. |
| `X \|\| Y` | *X* ∨ *Y*      | [Logical disjunction.](https://en.wikipedia.org/wiki/Logical_disjunction)<br />`X` and `Y` must be a relation. |

You can group a part of an expression or a relation with `(` … `)`.

## References

- [Tup96] Jeffrey Allen Tupper. *Graphing Equations with Generalized Interval Arithmetic.* M.Sc. Thesis, Department of Computer Science, University of Toronto, January 1996. http://www.dgp.toronto.edu/~mooncake/thesis.pdf

- [Tup01] Jeff Tupper. *Reliable Two-Dimensional Graphing Methods for Mathematical Formulae with Two Free Variables.* SIGGRAPH 2001 Conference Proceedings, August 2001. http://www.dgp.toronto.edu/~mooncake/papers/SIGGRAPH2001_Tupper.pdf
