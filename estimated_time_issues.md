# Estimated Time Calculation Issues in Timer Resolution Benchmark

## Overview
During development of the Timer Resolution Benchmark tool, we encountered a significant discrepancy between the initial "Estimated time" displayed at the start of the benchmark and the real-time "ETA" that appears during the execution.

## Problem Description

### Initial Estimated Time vs Real-time ETA Discrepancy
- **Initial Estimated Time:** Shows at the beginning based on calculation: `(number of points) * (estimated seconds per point) / 60`
- **Real-time ETA:** Calculated based on actual performance after measurements have started

### Example from Console Output
```console
⏱️  Estimated time: 24.0 minutes
⠁ [00:00:00] [----------------------------------------] 0/1000 0.5000 ms | ETA: 0s
Measuring 0.5000 ms (3 runs × 3 samples)
Verified: 0.5000 ms ✓.   Verified: 0.5000 ms ✓.   Verified: 0.5000 ms ✓. ✓
Mean(Δ): 0.4860 ms, P95(Δ): 0.4860 ms, MAD(Δ): 0.4860 ms, Outliers removed: 1
⠁ [00:00:06] [#---------------------------------------] 1/1000 0.5000 ms | ETA: 2h       Current best: 0.5000 ms (score=0.5000)
⠁ [00:00:06] [#---------------------------------------] 1/1000 0.5001 ms | ETA: 2h
Measuring 0.5001 ms (3 runs × 3 samples)
```

In this example:
- Initial estimate: **24.0 minutes**
- Real-time ETA after first measurement: **2 hours**

## Root Causes

### 1. Static Formula vs Dynamic Reality
The initial estimate uses a static formula:
```rust
let estimated_seconds_per_point = (params.sample_value as f64 / 25.0) * 12.0;
```
This doesn't account for:
- System-specific performance characteristics
- Current system load
- Process startup overhead
- Anti-virus interference
- Background processes

### 2. Incomplete Overhead Calculation
The calculation assumes each measurement takes approximately 12 seconds for every 25 samples, but doesn't consider:
- Process cleanup time between measurements (~300ms+)
- Sleep time between measurements (~600ms)
- Timer resolution setting time
- Process spawning time
- File I/O operations

### 3. Hardware and OS Variability
- Different CPUs may have different timer resolution characteristics
- Different system loads affect performance
- Different Windows versions may behave differently
- Background processes consuming resources

## Attempts at Resolution

### First Attempt
```rust
// Initial formula using 6.5 seconds per point
let estimated_seconds_per_point = (params.sample_value as f64 / 25.0) * 6.5;
```

### Second Attempt  
```rust
// Adjusted to 12 seconds per point to account for overhead
let estimated_seconds_per_point = (params.sample_value as f64 / 25.0) * 12.0;
```

Both attempts still resulted in significant discrepancies between initial estimate and real-time ETA.

## Why Real-time ETA is More Accurate

The real-time ETA uses actual performance data:
```rust
let elapsed = start_time.elapsed().as_secs_f64();
let avg_time_per_point = elapsed / (i as f64);
let remaining_points = total_points - i;
let eta_seconds = avg_time_per_point * (remaining_points as f64);
```

This provides a realistic estimate based on:
- Actual time taken for completed measurements
- Real system performance during execution
- Current system conditions

## Conclusion

The fundamental issue is that any initial estimate made before execution can never be as accurate as real-time calculations based on actual performance. The real-time ETA is and should remain the more reliable indicator of actual remaining time.

**Recommendation:** Consider the initial "Estimated time" as a rough approximation only, with the real-time "ETA" being the accurate indicator of remaining time during execution.