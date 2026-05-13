// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "MinnalKit",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(
            name: "MinnalKit",
            targets: ["MinnalKit"]
        ),
    ],
    targets: [
        .target(
            name: "MinnalKit",
            path: "Sources/MinnalKit"
        ),
        .executableTarget(
            name: "MinnalKitTests",
            dependencies: ["MinnalKit"],
            path: "Tests/MinnalKitTests"
        ),
    ]
)
