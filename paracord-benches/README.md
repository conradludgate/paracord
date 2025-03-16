Results

## Scoped

### Linux Ryzen 9 7950x

```
Timer precision: 9 ns
paracord          fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         63.9 ns       │ 525.3 ns      │ 78.39 ns      │ 83.37 ns
│  ├─ t=2         74.86 ns      │ 166.6 ns      │ 88.51 ns      │ 97.17 ns
│  ╰─ t=16        48.43 ns      │ 247.4 ns      │ 114.4 ns      │ 120.5 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         37.91 ns      │ 91.38 ns      │ 39.81 ns      │ 42.28 ns
│  ├─ t=2         45.21 ns      │ 409 ns        │ 328.2 ns      │ 240.1 ns
│  ╰─ t=16        192.7 ns      │ 1.964 µs      │ 1.633 µs      │ 1.584 µs
╰─ resolve                      │               │               │
   ├─ t=1         9.423 ns      │ 34.87 ns      │ 11.01 ns      │ 12.17 ns
   ├─ t=2         8.332 ns      │ 35.61 ns      │ 11.16 ns      │ 11.89 ns
   ╰─ t=16        9.002 ns      │ 38.09 ns      │ 13.15 ns      │ 15.27 ns

lasso             fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         52.08 ns      │ 420.6 ns      │ 76.05 ns      │ 79.82 ns
│  ├─ t=2         57.3 ns       │ 190.7 ns      │ 85.54 ns      │ 90.81 ns
│  ╰─ t=16        63.71 ns      │ 217.7 ns      │ 109.5 ns      │ 117.1 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         58.1 ns       │ 99.16 ns      │ 71.15 ns      │ 72.42 ns
│  ├─ t=2         342.6 ns      │ 606.6 ns      │ 515.4 ns      │ 514 ns  
│  ╰─ t=16        705.1 ns      │ 2.51 µs       │ 1.422 µs      │ 1.431 µs
╰─ resolve                      │               │               │
   ├─ t=1         26.1 ns       │ 59.19 ns      │ 30.28 ns      │ 31.01 ns
   ├─ t=2         28.71 ns      │ 128.7 ns      │ 47.67 ns      │ 51.45 ns
   ╰─ t=16        34.63 ns      │ 144.8 ns      │ 84.37 ns      │ 86.78 ns
```

### Apple M4 Max

```
Timer precision: 41 ns
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

### Linux Ryzen 9 7950x

```
Timer precision: 9 ns
ustr_global       fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         44.54 ns      │ 68.21 ns      │ 51.89 ns      │ 52.69 ns
│  ├─ t=2         47.84 ns      │ 97.97 ns      │ 56.56 ns      │ 57.39 ns
│  ╰─ t=16        83.58 ns      │ 392.9 ns      │ 152.3 ns      │ 160 ns  
├─ get_or_intern                │               │               │
│  ├─ t=1         61.84 ns      │ 2.913 µs      │ 83.51 ns      │ 114.1 ns
│  ├─ t=2         95.82 ns      │ 6.171 µs      │ 128.4 ns      │ 240.3 ns
│  ╰─ t=16        151.2 ns      │ 835.6 ns      │ 371.5 ns      │ 379 ns  
╰─ resolve                      │               │               │
   ├─ t=1         3.611 ns      │ 12.84 ns      │ 4.62 ns       │ 4.738 ns
   ├─ t=2         4.051 ns      │ 13.88 ns      │ 4.996 ns      │ 5.057 ns
   ╰─ t=16        4.471 ns      │ 31.73 ns      │ 7.106 ns      │ 8.012 ns

paracord_global   fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         39.63 ns      │ 83.95 ns      │ 45.28 ns      │ 47.39 ns
│  ├─ t=2         45.37 ns      │ 81.39 ns      │ 50.62 ns      │ 52.22 ns
│  ╰─ t=16        60.94 ns      │ 257.1 ns      │ 105.7 ns      │ 113.5 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         101.4 ns      │ 3.039 µs      │ 117.2 ns      │ 190.5 ns
│  ├─ t=2         296 ns        │ 4.648 µs      │ 347.6 ns      │ 359.1 ns
│  ╰─ t=16        199.8 ns      │ 16.01 µs      │ 1.23 µs       │ 1.851 µs
╰─ resolve                      │               │               │
   ├─ t=1         4.731 ns      │ 20.95 ns      │ 6.751 ns      │ 6.818 ns
   ├─ t=2         7.122 ns      │ 27.27 ns      │ 8.456 ns      │ 9.273 ns
   ╰─ t=16        7.031 ns      │ 36.54 ns      │ 9.936 ns      │ 11.3 ns 

lasso_global      fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         33.25 ns      │ 76.23 ns      │ 37.74 ns      │ 40.36 ns
│  ├─ t=2         35.26 ns      │ 74.71 ns      │ 39.28 ns      │ 40.58 ns
│  ╰─ t=16        52.24 ns      │ 209.7 ns      │ 104.4 ns      │ 110.3 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         161.2 ns      │ 26.96 µs      │ 200.6 ns      │ 438 ns  
│  ├─ t=2         282.8 ns      │ 595.7 ns      │ 358.7 ns      │ 392.2 ns
│  ╰─ t=16        528.9 ns      │ 57.91 µs      │ 2.01 µs       │ 3.63 µs 
╰─ resolve                      │               │               │
   ├─ t=1         47.31 ns      │ 147.4 ns      │ 53.88 ns      │ 56.33 ns
   ├─ t=2         51.26 ns      │ 125.6 ns      │ 68.98 ns      │ 69.9 ns 
   ╰─ t=16        65.62 ns      │ 195.5 ns      │ 102.7 ns      │ 107.4 ns
```

### Apple M4 Max

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
