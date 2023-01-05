# Cache for controlling and reducing IO calls

## Overview 

This caching library has been designed for specific use-cases where:

* getting a "fresh" value can be time consuming and can fail (eg. IOs with hardware)
* getting multiple values at once can be more efficient than getting each value independantly.

Typically, its primary use was to retrieve position/speed/temperature/etc from multiple motors using serial communication. In this setup, the motors are daisy chained, and in the protocol used to communicate with them, a specific message can be used to retrieve a register value for multiple motors at once.

Many other caching implementations exist than can better fit other need.

## License

This library is licensed under the Apache License 2.0.

## Support

It's developed and maintained by [Pollen-Robotics](https://pollen-robotics.com). They developped open-source tools for robotics manipulation.
Visit https://pollen-robotics.com to learn more or join our Dicord community if you have any questions or want to share your ideas. 
