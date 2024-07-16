import Vapor

private struct DecodingWrapper<T: Decodable>: Decodable {
  let decoder: Decoder

  init(from decoder: Decoder) throws {
    self.decoder = decoder
  }
}

extension Validatable {
  public static func validate<T: Encodable & Validatable>(_ value: T) throws {
    let encoder = JSONEncoder()
    let data = try encoder.encode(value)

    let decoder = JSONDecoder()
    let topLevelDecoder = try decoder.decode(
      DecodingWrapper<Components.Schemas.Login>.self, from: data
    ).decoder

    try T.validate(topLevelDecoder)
  }
}
