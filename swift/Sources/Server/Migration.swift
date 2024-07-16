import Fluent
import SQLKit

struct InitialMigration: AsyncMigration {
  func prepare(on database: Database) async throws {
    try await database.schema("users")
      .id()
      .field("email", .string, .required)
      .field("password", .string, .required)
      .field("created_at", .datetime, .required, .sql(.default(SQLFunction("now"))))
      .field("updated_at", .datetime, .required, .sql(.default(SQLFunction("now"))))
      .unique(on: "email")
      .create()
  }

  func revert(on database: Database) async throws {
    try await database.schema("users").delete()
  }
}
