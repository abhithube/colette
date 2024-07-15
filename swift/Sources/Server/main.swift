import OpenAPIRuntime
import OpenAPIVapor
import Vapor

struct API: APIProtocol {
  func login(_ input: Operations.login.Input) async throws -> Operations.login.Output {
    .ok(.init(body: .json(.init(id: "", email: "", createdAt: .now, updatedAt: .now))))
  }
}

let app = Vapor.Application()

let transport = VaporTransport(routesBuilder: app)

let api = API()

try api.registerHandlers(on: transport, serverURL: Servers.server1())

try await app.execute()