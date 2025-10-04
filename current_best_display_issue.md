# Current Best Display Issue in Timer Resolution Benchmark

## Overview
The "Current best" message is not consistently displayed during the linear search method execution, despite being correctly implemented in the code.

## Problem Description

### Expected Behavior
- "Current best" message should appear after every measurement
- Format: `Current best: X.XXXX ms (score=X.XXXX)`

### Actual Behavior
- "Current best" message appears correctly for the first several measurements (1-9)
- After measurement 10, the "Current best" message disappears from the console output
- The code to print the message is still running, but the output is not visible in the console

### Example from Console Output
```console
Mean(Δ): 0.4901 ms, P95(Δ): 0.4901 ms, MAD(Δ): 0.4901 ms, Outliers removed: 1
⠒ [00:00:30] [#---------------------------------------] 5/1000 0.5004 ms | ETA: 2h
       Current best: 0.5000 ms (score=0.5000)  ← VISIBLE
⠒ [00:00:30] [#---------------------------------------] 5/1000 0.5005 ms | ETA: 2h
Measuring 0.5005 ms (3 runs × 3 samples)

Mean(Δ): 0.5025 ms, P95(Δ): 0.5025 ms, MAD(Δ): 0.5025 ms, Outliers removed: 0
⠂ [00:00:36] [#---------------------------------------] 6/1000 0.5005 ms | ETA: 2h
       Current best: 0.5000 ms (score=0.5000)  ← VISIBLE
⠂ [00:00:36] [#---------------------------------------] 6/1000 0.5006 ms | ETA: 2h
Measuring 0.5006 ms (3 runs × 3 samples)

...

Mean(Δ): 0.7520 ms, P95(Δ): 0.7520 ms, MAD(Δ): 0.7520 ms, Outliers removed: 1
⠴ [00:01:00] [#---------------------------------------] 10/1000 0.5009 ms | ETA: 2h
⠴ [00:01:00] [#---------------------------------------] 10/1000 0.5010 ms | ETA: 2h  ← "Current best" MISSING
Measuring 0.5010 ms (3 runs × 3 samples)
```

## Root Cause Analysis

### Primary Cause
- The `indicatif` progress bar library uses terminal control sequences to update the progress bar efficiently
- After a certain number of updates, these control sequences may be overwriting or hiding lines where "Current best" is printed
- The progress bar updates are happening faster than the console can properly display both the progress bar and the "Current best" messages

### Contributing Factors
1. **Console buffering**: Output may be held in buffers that get overwritten
2. **Progress bar restoration**: The progress bar library may restore cursor positions and overwrite previous output
3. **Terminal control sequences**: Advanced terminal features used by the progress bar may interfere with normal output

## Attempted Solutions

### Solution 1: Adding extra newlines
- Added `println!();` before "Current best" message
- Result: Did not resolve the issue

### Solution 2: Output flushing
- Added `std::io::stdout().flush().unwrap();` after printing
- Result: Did not resolve the issue

### Solution 3: Timing considerations
- Code runs correctly in the loop after each measurement
- The issue is display-related, not logic-related

## Technical Details

### Code Implementation
```rust
// This code correctly runs after each measurement:
if !measurements.is_empty() {
    let current_best = measurements.iter()
        .min_by(|a, b| {
            let score_a = a.statistics.performance_score(&weights);
            let score_b = b.statistics.performance_score(&weights);
            score_a.partial_cmp(&score_b).unwrap()
        })
        .unwrap();
    println!("       {}", localization.get_current_best(
        current_best.resolution_ms, 
        current_best.statistics.performance_score(&weights)
    ));
    std::io::stdout().flush().unwrap();
}
```

## Potential Solutions for Future Investigation

1. **Use a different progress bar approach**: Consider pausing the progress bar briefly when displaying "Current best"
2. **Integrate "Current best" into the progress bar message**: Update the progress bar template to include the current best value
3. **Use a separate output channel**: Write "Current best" to stderr to avoid conflicts with progress bar on stdout
4. **Periodic display**: Show "Current best" every N measurements instead of every measurement
5. **Alternative progress indication**: Use a different progress library or approach that doesn't interfere with regular output

## Impact
- Users cannot track the optimization progress after the first few measurements
- Reduces the usability of the linear search method
- Makes it harder to decide when to stop the benchmark early if the optimum has been found