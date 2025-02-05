# EVE Fit OS

## Prerequisites

### Environments

- Protobuf Compiler: >= 29.3 (optional)
- Python environment: >= 3.12 (dev: 3.12.4)
- UV: >= 0.5.24
- Rust: >= 1.83.0

### Setup

1.  Compile protobuf for python (optional)
    (The repo offers a pre-compiled python file.)
    
    ```bash
    protoc efos.proto --python_out .
    # Output file: efos_pb2.py
    ```

2.  Download FSD files.  
    You have to manually download FSD datasets and unzip it to a separate folder (in this guide, `./fsd`). URL: [fsd.zip](https://eve-static-data-export.s3-eu-west-1.amazonaws.com/tranquility/fsd.zip).
3.  Convert FSD files to protobuf.
    1.  Initialize UV env
        
        ```bash
        uv sync
        ```
    
    2.  Run converter
        
        ```bash
        uv run -m data.convert <path/to/fsd> <path/to/patches> <path/to/output>
        ```

        Example:
        ```bash
        uv run -m data.convert data/fsd data/patches data/out
        ```
    
    3.  Wait until the command exits.
4.  Build the library

    ```bash
    cargo build
    ```

## Usage

### Create a fit object

```rust
use eve_fit_os::fit::{ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState, ItemCharge, ItemDrone},

let fit = ItemFit {
    ship_type_id: 628, // Arbitrator, 主宰级
    // ship modules, including rigs and subsystems.
    modules: (0..3)
        .map(|index| ItemModule {
            // Rapid Light Missile Launcher II
            // 轻型快速导弹发射器 II
            type_id: 1877,
            slot: ItemSlot {
                slot_type: ItemSlotType::High,
                index,
            },
            state: ItemState::Active,
            charge: Some(ItemCharge {
                // Mjolnir Fury Light Missile
                // 雷神愤怒轻型导弹
                type_id: 2613,
            })
        })
        .collect(),
    drones: vec![ItemDrone {
        type_id: 2456, // Hobgoblin II, 地精灵 II
        state: ItemState::Active,
    }],
    implants: vec![ItemImplant {
        // Zainou 'Snapshot' Light Missiles LM-905
        // 载诺 快照 轻型导弹 LM-905
        type_id: 27252
    }]
};
```

### Create character skills

```rust
let skills: BTreeMap<i32, u8> =
    [
        // ...
    ]
    .into_iter()
    .collect();
```

### Initialize data provider

Any type implements `InfoProvider` can be a data provider.

The lib itself offers a container initialized from the protobuf data mentioned above.

```rust
use eve_fit_os::protobuf::Database;

let info = Database::init("data/out/pb2").unwrap();
```

### Calculate a fit

```rust
use eve_fit_os::fit::FitContainer;
use eve_fit_os::calculate::calculate;

let container = FitContainer::new(fit, skills);
let out = calculate(&container, &info);
```

### Read some attributes

```rust
let dmg = out
    .modules
    .iter()
    .find(|t| t.type_id == 1877)
    .and_then(|t| t.charge.as_ref())
    .and_then(|t| t.attributes.get(&114)) // emDamage
    .and_then(|t| t.value)
    .unwrap_or_default();
assert!(dmg <= 161 && dmg >= 159); // around 160.4
```

## Appendix

### Patches

#### Why patches

To easily calculate some special attributes.

#### How patches are inserted

All patches' id are negative, starting from `-1`.
The sequence and exact id of a patch item is not guaranteed,
you have to check the patch file and the output json to figure out
what a patch item and its id is.

### Why proto2

This project's data files are not subjected to change unless the project itself changes.
That means we could assume that no fields will be added/removed,
so we can force some of the fields to be `required`, which is not allowed in proto3.
