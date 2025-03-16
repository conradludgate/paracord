Results

## Scoped

### Apple M4 Max

```
paracord          fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         22.28 ns      │ 53.11 ns      │ 25.86 ns      │ 27.29 ns
│  ├─ t=2         29.2 ns       │ 110.6 ns      │ 38.2 ns       │ 39.01 ns
│  ╰─ t=14        34.07 ns      │ 173.5 ns      │ 115.5 ns      │ 116.8 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         26.65 ns      │ 54.65 ns      │ 30.49 ns      │ 30.07 ns
│  ├─ t=2         102.5 ns      │ 198.5 ns      │ 128.4 ns      │ 129.7 ns
│  ╰─ t=14        1.039 µs      │ 2.268 µs      │ 1.465 µs      │ 1.508 µs
╰─ resolve                      │               │               │
   ├─ t=1         4.076 ns      │ 74.24 ns      │ 5.242 ns      │ 5.701 ns
   ├─ t=2         4.95 ns       │ 24.53 ns      │ 6.368 ns      │ 6.822 ns
   ╰─ t=14        5.367 ns      │ 134.2 ns      │ 6.618 ns      │ 9.706 ns

lasso             fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         19.48 ns      │ 57.4 ns       │ 23.21 ns      │ 24.49 ns
│  ├─ t=2         26.15 ns      │ 81.53 ns      │ 33.03 ns      │ 33.83 ns
│  ╰─ t=14        41.07 ns      │ 177.3 ns      │ 106.8 ns      │ 102.6 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         41.4 ns       │ 105.8 ns      │ 46.53 ns      │ 47.13 ns
│  ├─ t=2         86.73 ns      │ 612.4 ns      │ 170 ns        │ 176 ns
│  ╰─ t=14        2.46 µs       │ 3.32 µs       │ 3.063 µs      │ 3.047 µs
╰─ resolve                      │               │               │
   ├─ t=1         10.19 ns      │ 68.19 ns      │ 13.07 ns      │ 13.36 ns
   ├─ t=2         16.32 ns      │ 53.86 ns      │ 22.32 ns      │ 22.54 ns
   ╰─ t=14        22.32 ns      │ 182.9 ns      │ 135.6 ns      │ 128.8 ns
```

## Global

### Apple M4 Max

```
Timer precision: 41 ns
ustr_global       fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         26 ns         │ 124.5 ns      │ 29.71 ns      │ 30.72 ns
│  ├─ t=2         33.25 ns      │ 83.96 ns      │ 38.28 ns      │ 39.14 ns
│  ╰─ t=14        62.75 ns      │ 251.5 ns      │ 144.2 ns      │ 145.1 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         36.88 ns      │ 1.209 µs      │ 45.9 ns       │ 55.95 ns
│  ├─ t=2         46.46 ns      │ 1.815 µs      │ 57.17 ns      │ 91.18 ns
│  ╰─ t=14        135.8 ns      │ 498.6 ns      │ 303.7 ns      │ 305.9 ns
╰─ resolve                      │               │               │
   ├─ t=1         0.884 ns      │ 12.09 ns      │ 1.259 ns      │ 1.671 ns
   ├─ t=2         0.967 ns      │ 8.967 ns      │ 1.676 ns      │ 1.727 ns
   ╰─ t=14        1.051 ns      │ 26.63 ns      │ 1.634 ns      │ 2.54 ns

paracord_global   fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         21.15 ns      │ 88.36 ns      │ 25.11 ns      │ 26.17 ns
│  ├─ t=2         29.23 ns      │ 101.4 ns      │ 34.03 ns      │ 35.28 ns
│  ╰─ t=14        39.11 ns      │ 178.2 ns      │ 119.8 ns      │ 122 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         57.69 ns      │ 1.612 µs      │ 67.19 ns      │ 93.38 ns
│  ├─ t=2         116.2 ns      │ 4.003 µs      │ 125.6 ns      │ 137.1 ns
│  ╰─ t=14        1.149 µs      │ 8.198 µs      │ 1.402 µs      │ 1.617 µs
╰─ resolve                      │               │               │
   ├─ t=1         3.78 ns       │ 173.4 ns      │ 5.614 ns      │ 5.868 ns
   ├─ t=2         4.988 ns      │ 20.98 ns      │ 6.447 ns      │ 6.613 ns
   ╰─ t=14        5.28 ns       │ 122.4 ns      │ 6.572 ns      │ 9.12 ns

lasso_global      fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         19.45 ns      │ 46.9 ns       │ 23.03 ns      │ 24.03 ns
│  ├─ t=2         25.15 ns      │ 84.15 ns      │ 30.82 ns      │ 31.4 ns
│  ╰─ t=14        43.28 ns      │ 170.9 ns      │ 119.5 ns      │ 123.6 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         67.57 ns      │ 10.73 µs      │ 82.93 ns      │ 155.3 ns
│  ├─ t=2         161 ns        │ 514.6 ns      │ 178 ns        │ 181.4 ns
│  ╰─ t=14        2.03 µs       │ 25.2 µs       │ 2.994 µs      │ 3.561 µs
╰─ resolve                      │               │               │
   ├─ t=1         21.49 ns      │ 73.07 ns      │ 23.74 ns      │ 24.54 ns
   ├─ t=2         27.28 ns      │ 56.7 ns       │ 31.32 ns      │ 31.8 ns
   ╰─ t=14        31.99 ns      │ 189.1 ns      │ 140.4 ns      │ 137.7 ns
```
