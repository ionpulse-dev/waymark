import Foundation

/// A window string from `Localizable.strings`. The key is the English text, so a build without the
/// bundle (`swift run`) still shows the right words.
func L(_ key: String) -> String {
    NSLocalizedString(key, tableName: nil, bundle: .main, value: key, comment: "")
}

/// A window string with values, for example `L("%d writes", n)`.
func L(_ key: String, _ args: CVarArg...) -> String {
    String(format: L(key), arguments: args)
}

import WaymarkBarCore

/// SwiftUI also has a `Document`. In this app it is always the waymark document.
typealias Document = WaymarkBarCore.Document
