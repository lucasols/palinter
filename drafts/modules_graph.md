## circular_4

```mermaid
graph LR
    dep1-->dep2
    dep2-->dep3
    dep2-->dep4
    dep3-->dep2
    dep4-->dep2
    dep4-->dep5
    dep5-->dep3
```

## non_cyclic_graph

```mermaid
graph LR
    A-->B
    A-->C
    B-->D
    C-->D
```

## dag_5

```mermaid
graph LR
  D[Dropdown] --> T[Typings]
  D --> P[Popover] --> PL[PortalLayer]
  D --> UDV[uDVU]
  D --> UOCO[uOCO]
  UDV --> useTimout --> T
```

# Simple 6

```mermaid
flowchart LR
  Dep1 --> Dep2 & Dep4
  Dep2 --> Dep3
  Dep4 --> Dep2
```

# Simple 7

```mermaid
flowchart LR
  Dep1 --> Dep2 --> Dep3
  Dep1 --> Dep4 --> Dep5
```

# Circular 8

```mermaid
flowchart LR
  circular --> dep1
  dep1 --> dep2 & dep3
  dep2 --> circular
  dep3 --> dep4
```

# Circular 9

```mermaid
flowchart LR
  A --> B & C
    B --> D
    B --> E
    E --> F
    F --> B
    C --> D
    D --> 1
    1 --> 2
    2 --> 3
    3 --> 1
```
