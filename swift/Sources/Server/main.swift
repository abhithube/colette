import Fluent
import FluentPostgresDriver
import OpenAPIRuntime
import OpenAPIVapor
import Vapor

enum AppError: Error {
  case runtimeError(String)
}

extension Components.Schemas.Login: Validatable {
  static func validations(_ validations: inout Validations) {
    validations.add("email", as: String.self, is: .email)
    validations.add("password", as: String.self, is: !.empty)
  }
}

extension Components.Schemas.Register: Validatable {
  static func validations(_ validations: inout Validations) {
    validations.add("email", as: String.self, is: .email)
    validations.add("password", as: String.self, is: !.empty)
  }
}

struct API: APIProtocol {
  func login(_ input: Operations.login.Input) async throws -> Operations.login.Output {
    switch input.body {
    case .json(let body):
      do {
        try Components.Schemas.Login.validate(body)
      } catch let error as ValidationsError {
        return .unprocessableContent(.init(body: .json(.init(message: error.description))))
      }
    }

    return .ok(.init(body: .json(.init(id: "", email: "", createdAt: .now, updatedAt: .now))))
  }

  func register(_ input: Operations.register.Input) async throws -> Operations.register.Output {
    switch input.body {
    case .json(let body):
      do {
        try Components.Schemas.Register.validate(body)
      } catch let error as ValidationsError {
        return .unprocessableContent(.init(body: .json(.init(message: error.description))))
      }
    }

    return .ok(.init(body: .json(.init(id: "", email: "", createdAt: .now, updatedAt: .now))))
  }
}

guard let databaseURL = Environment.get("DATABASE_URL") else {
  throw AppError.runtimeError("\"DATABASE_URL\" not set")
}

let app = Vapor.Application()

app.databases.use(
  .postgres(
    configuration: try .init(url: databaseURL)
  ),
  as: .psql
)

app.migrations.add(InitialMigration())

try await app.autoMigrate().get()

let transport = VaporTransport(routesBuilder: app)

let api = API()

try api.registerHandlers(on: transport, serverURL: Servers.server1())

try await app.execute()
