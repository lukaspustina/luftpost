# luftpost

[![Linux & OS X Build Status](https://img.shields.io/travis/lukaspustina/luftpost.svg?label=Linux%20%26%20OS%20X%20Build%20Status)](https://travis-ci.org/lukaspustina/luftpost) [![Windows Build status](https://img.shields.io/appveyor/ci/lukaspustina/luftpost.svg?label=Windows%20Build%20Status)](https://ci.appveyor.com/project/lukaspustina/luftpost/branch/master) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg?label=License)](./LICENSE) [![](https://img.shields.io/crates/v/luftpost.svg)](https://crates.io/crates/luftpost)



Watches [luftdaten.info](http://luftdaten.info) particulates sensors and sends E-Mails if measurements exceed thresholds

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Table of Contents**

- [Todos](#todos)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Todos

* [X] Features

    * [X] Read measurement from multiple local sensors by id

    * [X] Clap: config file, just read measurements

    * [X] Read config file

        * [X] Sensors

        * [X] Threshold by sensor

        * [X] Default threshold

        * [X] E-Mail: receipient, subjects

        * [X] E-Mail Options: exceed, no-data, okay

    * [X] Print Measurement data to terminal

    * [X] Check thresholds

    * [X] Send E-Mails

        * [X] Send E-Mails

        * [X] HTML E-Mails with template engine

        * [X] Add public sensor ID and add graphs to HTML body

* [X] Infrastructure

    * [X] Travis CI, appveyor

    * [ ] Travis: Cross compile for ARM

    * [X] Travis: Build Debian packages (x86, ARM)

    * [X] Travis, Appveyor: Push binaries to GitHub realses

* [-] Milestone 1

    * [X] brew recipe

    * [X] Ansible Role

* [ ] Milestone 2

    * [ ] Show full error stack

[ ] Release 0.1

   * [ ] Readme


## Future Work

    * [ ] Progress bar for terminal operation

    * [ ] Send E-Mails: Move to Futures

