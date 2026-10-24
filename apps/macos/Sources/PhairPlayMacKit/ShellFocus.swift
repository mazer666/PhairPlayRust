import Foundation

/// A surface of the window that can hold the focus.
///
/// WHAT: the button of the banner of a failure and the four cards of the
/// protocols. Every surface that the user can reach with the keyboard is
/// one of these.
///
/// WHY: The window is built for the keyboard first, and the order of the
/// focus has to be a fact that can be written down and tested instead of
/// something that depends on the order in which the views happen to be
/// drawn. A surface that is not in this enumeration cannot be reached, and
/// a surface that is in it always shows the frame of the focus.
///
/// EXAMPLE: the focus sits on ``FocusTarget/card(_:)`` with the case
/// `.airplay` while the user looks at the card of AirPlay.
public enum FocusTarget: Hashable, Sendable {

    /// The button that closes the banner of a failure.
    case dismissFailure

    /// The card of one protocol.
    case card(ProtocolId)
}

/// The direction of a move inside the grid of the cards.
///
/// WHY: The four directions are the four arrow keys. They are an
/// enumeration and not four numbers, so that a switch over them cannot
/// forget a case.
public enum GridDirection: Sendable, CaseIterable {

    /// The key of the arrow to the left.
    case left

    /// The key of the arrow to the right.
    case right

    /// The key of the arrow up.
    case up

    /// The key of the arrow down.
    case down
}

/// The order of the focus of the window.
///
/// WHAT: the list of the surfaces that the keyboard walks through, and the
/// moves inside the grid of the cards.
///
/// WHY: The window has four cards in a grid of two columns and two rows,
/// and the user has to be able to walk through them with the tab key and
/// with the arrow keys without the mouse. The order of the focus is
/// therefore written down here, as data that can be tested, and the view
/// only draws it.
///
/// The tab key walks through the whole ring and comes back to the first
/// surface at the end. The arrow keys move inside the grid of the cards
/// and stop at the edge of the grid, because a move that leaves the grid
/// would land on a surface that has nothing to do with the direction of
/// the key.
///
/// EXAMPLE: the ring of a receiver without a failure holds the four cards
/// in the order of the protocols; the ring of a receiver with a failure
/// holds the button of the banner before them.
public struct FocusRing: Sendable, Equatable {

    /// The number of columns of the grid of the cards.
    ///
    /// WHY: The four cards of the receiver are shown as a grid of two
    /// columns and two rows, because a card is wide enough to carry the
    /// state of its protocol and because a grid of four cards fits into
    /// the window without a horizontal scroll bar.
    public static let columns = 2

    /// The surfaces of the window, in the order of the focus.
    public let targets: [FocusTarget]

    /// Creates a ring out of the surfaces of a window.
    ///
    /// - Parameter targets: the surfaces, in the order of the focus.
    public init(targets: [FocusTarget]) {
        self.targets = targets
    }

    /// Creates the ring of a window for a given state of the receiver.
    ///
    /// WHY: The button of the banner exists only when there is a failure,
    /// and the cards exist only for the protocols that the bridge reports.
    /// The order of the protocols is the order of the product and not the
    /// order in which the bridge happens to answer, so that the grid of
    /// the window does not jump around when a new answer arrives.
    ///
    /// - Parameter state: the state of the receiver.
    public init(state: ReceiverUIState) {
        var surfaces: [FocusTarget] = []
        if state.failure != nil {
            surfaces.append(.dismissFailure)
        }
        let reported = state.protocols.map(\.protocolId)
        for identifier in ProtocolId.allCases where reported.contains(identifier) {
            surfaces.append(.card(identifier))
        }
        self.init(targets: surfaces)
    }

    /// The cards of the grid, without the button of the banner.
    public var cards: [FocusTarget] {
        targets.filter { target in
            if case .card = target {
                return true
            }
            return false
        }
    }

    /// Whether the ring holds a surface.
    public func contains(_ target: FocusTarget) -> Bool {
        targets.contains(target)
    }

    /// The first surface of the ring.
    public var first: FocusTarget? {
        targets.first
    }

    /// The surface that follows the given one.
    ///
    /// - Parameter target: the surface that holds the focus, or `nil` when
    ///   nothing holds it yet.
    /// - Returns: the next surface, or `nil` when the ring is empty. The
    ///   focus comes back to the first surface at the end of the ring.
    public func next(after target: FocusTarget?) -> FocusTarget? {
        guard !targets.isEmpty else {
            return nil
        }
        guard let target, let index = targets.firstIndex(of: target) else {
            return targets.first
        }
        return targets[(index + 1) % targets.count]
    }

    /// The surface that precedes the given one.
    ///
    /// - Parameter target: the surface that holds the focus, or `nil` when
    ///   nothing holds it yet.
    /// - Returns: the previous surface, or `nil` when the ring is empty.
    ///   The focus comes back to the last surface at the beginning of the
    ///   ring.
    public func previous(before target: FocusTarget?) -> FocusTarget? {
        guard !targets.isEmpty else {
            return nil
        }
        guard let target, let index = targets.firstIndex(of: target) else {
            return targets.last
        }
        return targets[(index + targets.count - 1) % targets.count]
    }

    /// The surface in the given direction inside the grid of the cards.
    ///
    /// - Parameters:
    ///   - direction: the direction of the arrow key.
    ///   - target: the surface that holds the focus, or `nil` when nothing
    ///     holds it yet.
    /// - Returns: the surface in that direction, the given surface when
    ///   the edge of the grid is reached, or the first card when the
    ///   focus was not in the grid at all.
    public func move(_ direction: GridDirection, from target: FocusTarget?) -> FocusTarget? {
        let cards = cards
        guard !cards.isEmpty else {
            return nil
        }
        guard let target, let index = cards.firstIndex(of: target) else {
            return cards.first
        }
        let rows = (cards.count + Self.columns - 1) / Self.columns
        let row = index / Self.columns
        let column = index % Self.columns
        switch direction {
        case .left:
            return column > 0 ? cards[index - 1] : cards[index]
        case .right:
            return column + 1 < Self.columns && index + 1 < cards.count
                ? cards[index + 1]
                : cards[index]
        case .up:
            return row > 0 ? cards[index - Self.columns] : cards[index]
        case .down:
            return row + 1 < rows && index + Self.columns < cards.count
                ? cards[index + Self.columns]
                : cards[index]
        }
    }
}
