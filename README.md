# TPVFITPLAY

This is a simple commandline tool which take one or more FIT files as input and outputs the ride data from the FIT file into a TrainingPeaksVirtual (TPV) compatible broadcast format (`focus.json`). Each time new ride data is read from the FIT file, the `focus.json`is updated. 

## What is this good for? 

I use it as a testing tool for [TPVUI](https://github.com/wendlers/tpvui) and [TPVBC2HTTP](https://github.com/wendlers/tpvbc2http).

## How to use it?

Point it to one or more FIT files and to the location an output file ('focus.json'). E.g. like so:

```
tpvfitplay tests/ride_1.fit tests/ride_2.fit -o focus.json
```

Then point e.g. [TPVUI](https://github.com/wendlers/tpvui) to the full path where output file is located (e.g. `file:///MyDir/MyOtherDir`). You should now see the data in [TPVUI](https://github.com/wendlers/tpvui) getting updated.