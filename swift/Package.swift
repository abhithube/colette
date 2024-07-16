// swift-tools-version: 5.10
import PackageDescription

let package = Package(
  name: "colette",
  platforms: [
    .macOS(.v14)
  ],
  products: [
    .executable(name: "colette-server", targets: ["ColetteServer"])
  ],
  dependencies: [
    .package(url: "https://github.com/apple/swift-openapi-generator", from: "1.2.1"),
    .package(url: "https://github.com/apple/swift-openapi-runtime", from: "1.4.0"),
    .package(url: "https://github.com/swift-server/swift-openapi-vapor", from: "1.0.1"),
    .package(url: "https://github.com/vapor/fluent", from: "4.11.0"),
    .package(url: "https://github.com/vapor/fluent-postgres-driver", from: "2.9.2"),
    .package(url: "https://github.com/vapor/vapor", from: "4.102.1"),
  ],
  targets: [
    .executableTarget(
      name: "ColetteServer",
      dependencies: [
        .product(name: "Fluent", package: "fluent"),
        .product(name: "FluentPostgresDriver", package: "fluent-postgres-driver"),
        .product(name: "OpenAPIRuntime", package: "swift-openapi-runtime"),
        .product(name: "OpenAPIVapor", package: "swift-openapi-vapor"),
        .product(name: "Vapor", package: "vapor"),
      ],
      path: "Sources/Server",
      plugins: [
        .plugin(name: "OpenAPIGenerator", package: "swift-openapi-generator")
      ]
    )
  ]
)
