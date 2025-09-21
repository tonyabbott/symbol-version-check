symbol-version-check
====================

__symbol-version-check__ is a Linux command-line tool that analyzes ELF executables and shared libraries to verify 
their symbol version compatibility with target systems. It helps developers ensure their applications can run on older 
Linux distributions by checking if the dynamically linked symbols they use are available in the target system's library
versions.

## Purpose

* __Compatibility Validation__: Prevent runtime failures due to linking of incompatible symbol versions
* __Distribution Targeting__: Can be integrated into build pipelines to catch compatibility issues early 

## How it works

1. Parses ELF files to extract dynamically linked undefined symbol information
2. Compares symbol versions against a specified maximum allowed version
3. Reports any symbols where the required version exceeds the maximum allowed version, indicating potential 
compatibility issues

## Usage

`Usage: symbol-version-check [OPTIONS] -m <max_version> <FILES>...`

For example, to help ensure your application is able to run on RHEL 7, which ships with glibc 2.17, you may run:
```
$ ./symbol-version-check -m GLIBC_2.17 my-application
my-application: FAIL
    pthread_attr_getstack@GLIBC_2.34 (libc.so.6)
    pthread_attr_getstacksize@GLIBC_2.34 (libc.so.6)
    pthread_create@GLIBC_2.34 (libc.so.6)
    pthread_getattr_np@GLIBC_2.32 (libc.so.6)
    pthread_key_create@GLIBC_2.34 (libc.so.6)
    pthread_setspecific@GLIBC_2.34 (libc.so.6)
    pthread_sigmask@GLIBC_2.32 (libc.so.6)
```

### Exit Codes

| Exit Code | Description                                                          |
|-----------|----------------------------------------------------------------------|
| **0**     | All files passed the check                                           |
| **1**     | Error checking one or more of the files                              |
| **2**     | Usage error                                                          |
| **3**     | One or more of the files referenced symbols with disallowed versions |

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in symbol-version-check by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.