import OpenAPIRuntime
import OpenAPIVapor
import Vapor

extension Components.Schemas.Login: Validatable {
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
}

let app = Vapor.Application()

let transport = VaporTransport(routesBuilder: app)

let api = API()

try api.registerHandlers(on: transport, serverURL: Servers.server1())

try await app.execute()
