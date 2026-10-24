import XCTest
@testable import PhairPlayMacKit

/// The tests of the order of the focus.
///
/// WHAT: the proof that the tab key walks through the cards of the grid in
/// the order of the product, that the shift tab key walks back, and that
/// the arrow keys move inside the grid and stop at its edge.
///
/// WHY: The window is operated without a mouse. The order of the focus is
/// therefore a promise to the user — "the next card is always the one
/// below the last one" — and a promise that is not written down and not
/// tested is a promise that breaks with the next change of the layout.
///
/// EXAMPLE: the tab key on the card of AirPlay goes to the card of
/// Miracast, and the arrow key down on the card of AirPlay goes to the
/// card of Google Cast.
final class ShellFocusTests: XCTestCase {

    func testTheRingHoldsTheFourCardsInTheOrderOfTheProduct() {
        let ring = FocusRing(state: state())
        XCTAssertEqual(ring.targets, [
            .card(.airplay),
            .card(.miracast),
            .card(.cast),
            .card(.dlna),
        ])
    }

    func testTheRingHoldsTheButtonOfTheBannerBeforeTheCards() {
        let ring = FocusRing(state: state(failure: true))
        XCTAssertEqual(ring.targets.first, .dismissFailure)
        XCTAssertEqual(ring.cards.count, 4)
    }

    func testTheRingHoldsNoButtonWithoutAFailure() {
        let ring = FocusRing(state: state(failure: false))
        XCTAssertFalse(ring.contains(.dismissFailure))
    }

    func testTheRingHoldsOnlyTheProtocolsThatTheBridgeReports() {
        var receiverState = state()
        receiverState.protocols = Array(receiverState.protocols.prefix(2))
        let ring = FocusRing(state: receiverState)
        XCTAssertEqual(ring.targets, [.card(.airplay), .card(.miracast)])
    }

    func testTheTabKeyGoesToTheNextCard() {
        let ring = FocusRing(state: state())
        XCTAssertEqual(ring.next(after: .card(.airplay)), .card(.miracast))
        XCTAssertEqual(ring.next(after: .card(.miracast)), .card(.cast))
    }

    func testTheTabKeyComesBackToTheFirstCardAtTheEnd() {
        let ring = FocusRing(state: state())
        XCTAssertEqual(ring.next(after: .card(.dlna)), .card(.airplay))
    }

    func testTheShiftTabKeyGoesToThePreviousCard() {
        let ring = FocusRing(state: state())
        XCTAssertEqual(ring.previous(before: .card(.miracast)), .card(.airplay))
        XCTAssertEqual(ring.previous(before: .card(.airplay)), .card(.dlna))
    }

    func testTheTabKeyStartsAtTheFirstSurfaceWhenNothingHoldsTheFocus() {
        let ring = FocusRing(state: state())
        XCTAssertEqual(ring.next(after: nil), .card(.airplay))
        XCTAssertEqual(ring.previous(before: nil), .card(.dlna))
    }

    func testTheArrowKeysMoveInsideTheGrid() {
        let ring = FocusRing(state: state())
        XCTAssertEqual(ring.move(.right, from: .card(.airplay)), .card(.miracast))
        XCTAssertEqual(ring.move(.left, from: .card(.miracast)), .card(.airplay))
        XCTAssertEqual(ring.move(.down, from: .card(.airplay)), .card(.cast))
        XCTAssertEqual(ring.move(.up, from: .card(.cast)), .card(.airplay))
        XCTAssertEqual(ring.move(.down, from: .card(.miracast)), .card(.dlna))
    }

    func testTheArrowKeysStopAtTheEdgeOfTheGrid() {
        let ring = FocusRing(state: state())
        XCTAssertEqual(ring.move(.left, from: .card(.airplay)), .card(.airplay))
        XCTAssertEqual(ring.move(.up, from: .card(.airplay)), .card(.airplay))
        XCTAssertEqual(ring.move(.right, from: .card(.miracast)), .card(.miracast))
        XCTAssertEqual(ring.move(.down, from: .card(.dlna)), .card(.dlna))
        XCTAssertEqual(ring.move(.left, from: .card(.cast)), .card(.cast))
    }

    func testAnArrowKeyLeavesTheBannerAndEntersTheGrid() {
        let ring = FocusRing(state: state(failure: true))
        XCTAssertEqual(ring.move(.right, from: .dismissFailure), .card(.airplay))
    }

    func testTheRingIsEmptyWithoutAnyProtocol() {
        var receiverState = state()
        receiverState.protocols = []
        let ring = FocusRing(state: receiverState)
        XCTAssertTrue(ring.targets.isEmpty)
        XCTAssertNil(ring.next(after: nil))
        XCTAssertNil(ring.move(.right, from: nil))
    }

    /// A state of the receiver with the four cards.
    private func state(failure: Bool = false) -> ReceiverUIState {
        ReceiverUIState(
            running: true,
            deviceName: "Attic",
            version: "0.1.0",
            protocols: ProtocolId.allCases.map { identifier in
                ProtocolCardState(
                    protocolId: identifier,
                    phase: .advertising,
                    enabled: true,
                    tcpPort: 7000,
                    udpPort: nil,
                    peerCount: 0,
                    lastError: nil
                )
            },
            nowPlaying: nil,
            failure: failure
                ? BridgeFailure(code: 1, details: "unknown protocol")
                : nil
        )
    }
}
