Results

## Scoped

### Linux Ryzen 9 7950x

```
Timer precision: 9 ns
paracord          fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         48.28 ns      │ 86.93 ns      │ 53.66 ns      │ 55.31 ns
│  ├─ t=2         56.3 ns       │ 152.2 ns      │ 67.6 ns       │ 72.78 ns
│  ╰─ t=16        49.26 ns      │ 147 ns        │ 60.57 ns      │ 65.57 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         68.58 ns      │ 151.4 ns      │ 79.13 ns      │ 80.74 ns
│  ├─ t=2         274.2 ns      │ 657 ns        │ 482.3 ns      │ 477.7 ns
│  ╰─ t=16        2.816 µs      │ 10.41 µs      │ 9.487 µs      │ 9.369 µs
╰─ resolve                      │               │               │
   ├─ t=1         13.11 ns      │ 80.79 ns      │ 13.87 ns      │ 19.44 ns
   ├─ t=2         13.89 ns      │ 88.11 ns      │ 16.04 ns      │ 21.69 ns
   ╰─ t=16        14.14 ns      │ 225.6 ns      │ 22.06 ns      │ 29.03 ns

lasso             fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         29.96 ns      │ 74.81 ns      │ 37.5 ns       │ 39.57 ns
│  ├─ t=2         52.73 ns      │ 207.1 ns      │ 61.84 ns      │ 64.95 ns
│  ╰─ t=16        55.65 ns      │ 165.2 ns      │ 88.97 ns      │ 91.64 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         58.17 ns      │ 108.5 ns      │ 61.43 ns      │ 62.38 ns
│  ├─ t=2         149 ns        │ 559.6 ns      │ 497.7 ns      │ 492.9 ns
│  ╰─ t=16        904.8 ns      │ 2.594 µs      │ 1.322 µs      │ 1.346 µs
╰─ resolve                      │               │               │
   ├─ t=1         14.51 ns      │ 44.21 ns      │ 20.66 ns      │ 21.63 ns
   ├─ t=2         19.59 ns      │ 76.44 ns      │ 43.09 ns      │ 37.9 ns
   ╰─ t=16        27.45 ns      │ 146.3 ns      │ 74.43 ns      │ 74.61 ns
```

### Apple M4 Max

```
Timer precision: 41 ns
paracord          fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         26.32 ns      │ 48.98 ns      │ 28.78 ns      │ 30.05 ns
│  ├─ t=2         27.61 ns      │ 49.36 ns      │ 33.94 ns      │ 34.19 ns
│  ╰─ t=14        36.94 ns      │ 147.5 ns      │ 42.3 ns       │ 52.44 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         49.78 ns      │ 83.53 ns      │ 58.88 ns      │ 59.04 ns
│  ├─ t=2         88.48 ns      │ 235.2 ns      │ 159.5 ns      │ 154.8 ns
│  ╰─ t=14        430 ns        │ 5.137 µs      │ 3.391 µs      │ 3.111 µs
╰─ resolve                      │               │               │
   ├─ t=1         1.822 ns      │ 8.947 ns      │ 2.488 ns      │ 3.193 ns
   ├─ t=2         2.947 ns      │ 9.072 ns      │ 4.155 ns      │ 4.612 ns
   ╰─ t=14        3.822 ns      │ 39.61 ns      │ 5.03 ns       │ 6.263 ns

lasso             fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         16.57 ns      │ 32.23 ns      │ 18.78 ns      │ 19.99 ns
│  ├─ t=2         22.36 ns      │ 72.86 ns      │ 28.96 ns      │ 29.99 ns
│  ╰─ t=14        38.98 ns      │ 171.7 ns      │ 117.5 ns      │ 117.2 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         35.44 ns      │ 52.69 ns      │ 37.32 ns      │ 38.64 ns
│  ├─ t=2         146.3 ns      │ 233.8 ns      │ 167.3 ns      │ 167.8 ns
│  ╰─ t=14        1.838 µs      │ 3.226 µs      │ 2.913 µs      │ 2.894 µs
╰─ resolve                      │               │               │
   ├─ t=1         9.363 ns      │ 16.73 ns      │ 11.25 ns      │ 11.64 ns
   ├─ t=2         17.11 ns      │ 29.15 ns      │ 21.53 ns      │ 21.46 ns
   ╰─ t=14        20.28 ns      │ 182.6 ns      │ 135.5 ns      │ 130.6 ns
```

## Global

### Linux Ryzen 9 7950x

```
Timer precision: 9 ns
ustr_global       fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         38.33 ns      │ 70.2 ns       │ 43.58 ns      │ 44.13 ns
│  ├─ t=2         41.8 ns       │ 108.5 ns      │ 54.28 ns      │ 57 ns
│  ╰─ t=16        56.61 ns      │ 165.5 ns      │ 82.55 ns      │ 86.46 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         47.13 ns      │ 1.708 µs      │ 61.44 ns      │ 79.36 ns
│  ├─ t=2         97.28 ns      │ 3.641 µs      │ 119 ns        │ 180.1 ns
│  ╰─ t=16        104.1 ns      │ 557 ns        │ 288.6 ns      │ 294.6 ns
╰─ resolve                      │               │               │
   ├─ t=1         2.473 ns      │ 11.9 ns       │ 3.323 ns      │ 3.45 ns
   ├─ t=2         3.112 ns      │ 12.67 ns      │ 3.843 ns      │ 4.125 ns
   ╰─ t=16        2.893 ns      │ 19.24 ns      │ 3.733 ns      │ 3.983 ns

paracord_global   fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         47.17 ns      │ 85.09 ns      │ 52.55 ns      │ 55.03 ns
│  ├─ t=2         52.75 ns      │ 209.2 ns      │ 69.33 ns      │ 78.73 ns
│  ╰─ t=16        48.64 ns      │ 161.2 ns      │ 63.47 ns      │ 66 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         76.94 ns      │ 6.128 µs      │ 93.71 ns      │ 217.5 ns
│  ├─ t=2         280.4 ns      │ 18.46 µs      │ 444.6 ns      │ 968 ns  
│  ╰─ t=16        7.192 µs      │ 9.957 µs      │ 8.979 µs      │ 8.951 µs
╰─ resolve                      │               │               │
   ├─ t=1         1.623 ns      │ 15.38 ns      │ 3.897 ns      │ 4.835 ns
   ├─ t=2         2.793 ns      │ 22.31 ns      │ 6.988 ns      │ 7.088 ns
   ╰─ t=16        2.413 ns      │ 24.49 ns      │ 5.193 ns      │ 5.601 ns

lasso_global      fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         31.82 ns      │ 70.73 ns      │ 36.5 ns       │ 39.57 ns
│  ├─ t=2         39.79 ns      │ 125 ns        │ 63.67 ns      │ 66.83 ns
│  ╰─ t=16        49.03 ns      │ 158.3 ns      │ 88.62 ns      │ 89.94 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         131.2 ns      │ 32.12 µs      │ 143.2 ns      │ 305.2 ns
│  ├─ t=2         409.6 ns      │ 578.2 ns      │ 473.2 ns      │ 473.2 ns
│  ╰─ t=16        304.1 ns      │ 38.19 µs      │ 1.604 µs      │ 3.028 µs
╰─ resolve                      │               │               │
   ├─ t=1         29.34 ns      │ 78.06 ns      │ 32.2 ns       │ 35.05 ns
   ├─ t=2         33.49 ns      │ 87.39 ns      │ 38.16 ns      │ 40.44 ns
   ╰─ t=16        34.29 ns      │ 150.1 ns      │ 88.29 ns      │ 89.68 ns
```

### Apple M4 Max

```
Timer precision: 41 ns
ustr_global       fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         21.03 ns      │ 31.19 ns      │ 22.98 ns      │ 23.11 ns
│  ├─ t=2         27.03 ns      │ 42.57 ns      │ 32.11 ns      │ 32.08 ns
│  ╰─ t=14        35.15 ns      │ 243.2 ns      │ 139.8 ns      │ 140.8 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         29.9 ns       │ 879.4 ns      │ 35.92 ns      │ 45.82 ns
│  ├─ t=2         48.11 ns      │ 2.449 µs      │ 58.15 ns      │ 94.27 ns
│  ╰─ t=14        183.9 ns      │ 459.4 ns      │ 295.2 ns      │ 295.8 ns
╰─ resolve                      │               │               │
   ├─ t=1         0.78 ns       │ 3.739 ns      │ 1.072 ns      │ 1.172 ns
   ├─ t=2         1.155 ns      │ 2.78 ns       │ 1.78 ns       │ 1.842 ns
   ╰─ t=14        1.03 ns       │ 22.82 ns      │ 1.613 ns      │ 2.181 ns

paracord_global   fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         26.15 ns      │ 70.61 ns      │ 28.44 ns      │ 29.72 ns
│  ├─ t=2         26.98 ns      │ 231.3 ns      │ 34.03 ns      │ 34.94 ns
│  ╰─ t=14        37.48 ns      │ 122.2 ns      │ 43.13 ns      │ 51.95 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         50.11 ns      │ 1.659 µs      │ 61.73 ns      │ 104.2 ns
│  ├─ t=2         140.9 ns      │ 7.013 µs      │ 271.3 ns      │ 518.9 ns
│  ╰─ t=14        698.4 ns      │ 6.122 µs      │ 4.22 µs       │ 3.996 µs
╰─ resolve                      │               │               │
   ├─ t=1         2.155 ns      │ 15.07 ns      │ 2.822 ns      │ 3.493 ns
   ├─ t=2         2.864 ns      │ 9.864 ns      │ 4.03 ns       │ 4.557 ns
   ╰─ t=14        3.864 ns      │ 31.15 ns      │ 5.03 ns       │ 5.947 ns

lasso_global      fastest       │ slowest       │ median        │ mean
├─ get                          │               │               │
│  ├─ t=1         18.32 ns      │ 43.57 ns      │ 20.86 ns      │ 21.86 ns
│  ├─ t=2         20.53 ns      │ 52.44 ns      │ 26.88 ns      │ 27.08 ns
│  ╰─ t=14        34.57 ns      │ 167.9 ns      │ 119.7 ns      │ 119.8 ns
├─ get_or_intern                │               │               │
│  ├─ t=1         63.36 ns      │ 9.664 µs      │ 69.48 ns      │ 132.6 ns
│  ├─ t=2         167.4 ns      │ 530.2 ns      │ 176.1 ns      │ 179.1 ns
│  ╰─ t=14        2.142 µs      │ 18.06 µs      │ 2.933 µs      │ 3.355 µs
╰─ resolve                      │               │               │
   ├─ t=1         20.57 ns      │ 49.78 ns      │ 22.73 ns      │ 23.38 ns
   ├─ t=2         27.44 ns      │ 42.4 ns       │ 31.73 ns      │ 31.42 ns
   ╰─ t=14        40.78 ns      │ 187.4 ns      │ 138.5 ns      │ 135.3 ns
```
