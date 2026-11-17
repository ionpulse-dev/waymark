// swift-tools-version:5.10
// The menu bar app of waymark (see specs/features/menu-bar-app).
import PackageDescription

let package = Package(
    name: "WaymarkBar",
    defaultLocalization: "en",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "WaymarkBar", targets: ["WaymarkBar"]),
        .library(name: "WaymarkBarCore", targets: ["WaymarkBarCore"]),
    ],
    targets: [
        .target(name: "WaymarkBarCore"),
        .executableTarget(
            name: "WaymarkBar",
            dependencies: ["WaymarkBarCore"],
            resources: [.process("Resources")]
        ),
        .testTarget(name: "WaymarkBarCoreTests", dependencies: ["WaymarkBarCore"]),
    ]
)
