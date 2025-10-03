//! Localization keys for UI elements

/// Localization keys for UI elements
#[derive(Debug, Clone, Copy)]
pub enum LocalizationKey {
    // Main title
    Title,
    // System Information section
    SystemInfo,
    WorkingDir,
    AdminPrivileges,
    WindowsVersion,
    Cpu,
    // System Configuration section
    SystemConfig,
    HpetStatus,
    // Benchmark Parameters section
    BenchmarkParams,
    StartValue,
    IncrementValue,
    EndValue,
    SampleValue,
    Iterations,
    // Dependencies section
    Dependencies,
    Found,
    MissingDeps,
    // Benchmark phases
    RobustOptimization,
    Parameters,
    Range,
    IterationsCount,
    RunsPerPoint,
    SamplesPerRun,
    Weights,
    Accuracy,
    Stability,
    WorstCase,
    Phase1,
    Phase2,
    Phase3,
    Points,
    Measurement,
    CurrentBest,
    // TOPSIS results
    TopsisRanking,
    TopResults,
    Rank,
    OptimalValue,
    OptimalRecommendation,
    // Executable names
    MeasureSleepExe,
    SetTimerResolutionExe,
    // Progress messages
    PressEnter,
    EnterNewValue,
    KeepCurrent,
    // Results saving
    ResultsSaved,
    // Exit messages
    ExitPrompt,
    BenchmarkComplete,
    WarningCleanup,
    // Optimization method selection
    OptimizationMethod,
    AvailableMethods,
    LinearMethod,
    LinearMethodDesc1,
    LinearMethodDesc2,
    LinearMethodDesc3,
    LinearMethodDesc4,
    HybridMethod,
    HybridMethodDesc1,
    HybridMethodDesc2,
    HybridMethodDesc3,
    MethodChoice,
    IterationsLinear,
    IterationsHybrid,
    IncrementNotUsed,
    // MeasureSleep test
    MeasureSleepTest,
    // Windows 10/11 changes
    WindowsChangesTitle,
    WindowsChangesPerProcess,
    WindowsChangesOwnResolution,
    WindowsChangesSetAffects,
    WindowsChangesMinimized,
    WindowsChangesProblem,
    WindowsChangesSeparateProcess,
    WindowsChangesCannotSee,
    WindowsChangesSolution,
    WindowsChangesGlobalResolution,
    WindowsChangesLowLevelApi,
    // Linear method title
    LinearMethodTitle,
    // Linear method parameters section
    LinearMethodParameters,
    LinearMethodRange,
    LinearMethodStep,
    LinearMethodPoints,
    LinearMethodRuns,
    LinearMethodSamples,
    LinearMethodEstimatedTime,
    // Linear method completion
    LinearMethodCompleted,
    LinearMethodPointsChecked,
    LinearMethodUnique,
    // Unique points message
    UniquePointsMessage,
    // Test measurement message
    TestMeasurementMessage,
}