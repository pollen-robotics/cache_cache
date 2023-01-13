# Cache for controlling and reducing IO calls

[![Build Status]][actions] [![Latest Version]][crates.io]

[Build Status]: https://img.shields.io/github/actions/workflow/status/pollen-robotics/cache_cache/rust.yml?branch=master
[actions]: https://github.com/pollen-robotics/cache_cache/actions?query=branch%3Amaster

[Latest Version]: https://img.shields.io/crates/v/cache_cache.svg
[crates.io]: https://crates.io/crates/cache_cache

## Overview 

This caching library has been designed for specific use-cases where:

* getting a "fresh" value can be time consuming and can fail (eg. IOs with hardware)
* getting multiple values at once can be more efficient than getting each value independantly.

Typically, its primary use was to retrieve position/speed/temperature/etc from multiple motors using serial communication. In this setup, the motors are daisy chained, and in the protocol used to communicate with them, a specific message can be used to retrieve a register value for multiple motors at once.

Many other caching implementations exist than can better fit other need.

## Documentation

### Example

```rust
use cache_cache::Cache;
use std::{error::Error, time::Duration};

fn get_position(ids: &[u8]) -> Result<Vec<f64>, Box<dyn Error>> {
    // For simplicity, this function always work.
    // But it's a mockup for a real world scenario where hardware IO can fail.
    Ok(ids.iter().map(|&id| id as f64 * 10.0).collect())
}

fn main() {
    let mut present_position = Cache::with_expiry_duration(Duration::from_millis(10));

    present_position.insert(10, 0.0);

    let pos = present_position
        .entries(&[10, 11, 12])
        .or_try_insert_with(get_position);

    assert!(pos.is_ok());
    assert_eq!(pos.unwrap(), vec![&0.0, &110.0, &120.0]);
}
```

See https://docs.rs/cache_cache for more information on APIs and examples.

## License

This library is licensed under the Apache License 2.0.

## Support

It's developed and maintained by [Pollen-Robotics](https://pollen-robotics.com). They developped open-source tools for robotics manipulation.
Visit https://pollen-robotics.com to learn more or join our Dicord community if you have any questions or want to share your ideas. 
