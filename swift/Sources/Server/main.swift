import Fluent
import FluentPostgresDriver
import OpenAPIRuntime
import OpenAPIVapor
import Vapor

enum AppError: Error {
  case runtimeError(String)
}

final class User: Model {
  static let schema = "users"

  @ID(key: .id)
  var id: UUID?

  @Field(key: "email")
  var email: String

  @Field(key: "password")
  var password: String

  @Timestamp(key: "created_at", on: .create)
  var createdAt: Date?

  @Timestamp(key: "updated_at", on: .update)
  var updatedAt: Date?

  init() {}

  init(
    id: UUID? = nil,
    email: String,
    password: String,
    createdAt: Date? = nil,
    updatedAt: Date? = nil
  ) {
    self.id = id
    self.email = email
    self.password = password
    self.createdAt = createdAt
    self.updatedAt = updatedAt
  }
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

extension Components.Schemas.User {
  init(from user: User) {
    self.init(
      id: user.id!.uuidString,
      email: user.email,
      createdAt: user.createdAt!,
      updatedAt: user.updatedAt!
    )
  }
}

struct API: APIProtocol {
  let database: any Database
  let paswordHasher: AsyncPasswordHasher

  func login(_ input: Operations.login.Input) async throws -> Operations.login.Output {
    switch input.body {
    case .json(let body):
      do {
        try Components.Schemas.Login.validate(body)

        let unauthorized = Operations.login.Output.unauthorized(
          .init(body: .json(.init(message: "Invalid credentials"))))

        guard
          let user = try await User.query(on: self.database).filter(\.$email == body.email).first()
        else {
          return unauthorized
        }

        let matches = try await self.paswordHasher.verify(body.password, created: user.password)
        if !matches {
          return unauthorized
        }
      } catch let error as ValidationsError {
        return .unprocessableContent(.init(body: .json(.init(message: error.description))))
      } catch {
        return .undocumented(statusCode: 500, .init())
      }
    }

    return .ok(.init(body: .json(.init(id: "", email: "", createdAt: .now, updatedAt: .now))))
  }

  func register(_ input: Operations.register.Input) async throws -> Operations.register.Output {
    switch input.body {
    case .json(let body):
      do {
        try Components.Schemas.Register.validate(body)

        let digest = try await self.paswordHasher.hash(body.password)

        let user = User(email: body.email, password: digest)
        try await user.create(on: self.database)

        return .ok(.init(body: .json(.init(from: user))))
      } catch let error as ValidationsError {
        return .unprocessableContent(.init(body: .json(.init(message: error.description))))
      } catch let error as PostgresError where error.code == .uniqueViolation {
        return .conflict(
          .init(body: .json(.init(message: "User already exists with email: \(body.email)"))))
      } catch {
        return .undocumented(statusCode: 500, .init())
      }
    }
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

app.passwords.use(.bcrypt)

let transport = VaporTransport(routesBuilder: app)

let api = API(database: app.db(.psql), paswordHasher: app.password.async)

try api.registerHandlers(on: transport, serverURL: Servers.server1())

try await app.execute()
