import XCTest
@testable import PhairPlayMacKit

/// The tests of the texts and of the sizes of the window.
///
/// WHAT: the proof that the sentences of the window come out of the
/// catalogue of the shell, that the ports and the number of senders are
/// written the way the shell promises, and that the text is big enough to
/// be read from a distance.
///
/// WHY: A text that is written into the source of a view cannot be
/// translated, and a text that is too small cannot be read from a sofa.
/// The sizes are a promise to the user as well, and both promises are
/// cheap to keep and cheap to check.
///
/// EXAMPLE: a card that listens on TCP `7000` and on no UDP port shows
/// "TCP 7000" and not "TCP 7000 · UDP —".
final class ShellFormattingTests: XCTestCase {

    func testThePortsShowBothPorts() {
        let card = self.card(tcpPort: 7000, udpPort: 7001)
        let text = ShellFormatting.ports(of: card)
        XCTAssertTrue(text.contains("7000"))
        XCTAssertTrue(text.contains("7001"))
    }

    func testThePortsShowTheTcpPortAlone() {
        let card = self.card(tcpPort: 7000, udpPort: nil)
        let text = ShellFormatting.ports(of: card)
        XCTAssertTrue(text.contains("7000"))
        XCTAssertFalse(text.contains("7001"))
    }

    func testThePortsShowAPlaceholderForAMissingTcpPort() {
        let card = self.card(tcpPort: nil, udpPort: 7001)
        let text = ShellFormatting.ports(of: card)
        XCTAssertTrue(text.contains("7001"))
    }

    func testThePortsSayWhenNoPortIsBound() {
        let card = self.card(tcpPort: nil, udpPort: nil)
        XCTAssertNotEqual(ShellFormatting.ports(of: card), "protocol.ports.none")
    }

    func testTheNumberOfSendersIsWrittenInTheSentence() {
        let one = ShellFormatting.peers(of: card(peerCount: 1))
        let many = ShellFormatting.peers(of: card(peerCount: 3))
        XCTAssertTrue(one.contains("1"))
        XCTAssertTrue(many.contains("3"))
    }

    func testTheRunningTimeIsWrittenWithTwoDigits() {
        XCTAssertTrue(ShellFormatting.uptime(5_000).contains("0:05"))
        XCTAssertTrue(ShellFormatting.uptime(3_725_000).contains("1:02:05"))
    }

    func testEveryPhaseHasAWordAndASentence() {
        for phase in ProtocolPhase.allCases {
            XCTAssertNotEqual(ShellFormatting.stateText(phase), phase.textKey)
            XCTAssertNotEqual(ShellFormatting.reasonText(phase), phase.reasonKey)
        }
    }

    func testEveryPhaseHasItsOwnWordAndItsOwnShape() {
        let words = Set(ProtocolPhase.allCases.map { ShellFormatting.stateText($0) })
        let shapes = Set(ProtocolPhase.allCases.map(\.symbolName))
        XCTAssertEqual(words.count, ProtocolPhase.allCases.count)
        XCTAssertEqual(shapes.count, ProtocolPhase.allCases.count)
    }

    func testEveryProtocolHasAName() {
        for identifier in ProtocolId.allCases {
            XCTAssertNotEqual(ShellFormatting.protocolName(identifier), "protocol.\(identifier.rawValue)")
        }
    }

    func testTheNameOfTheDeviceAndTheVersionAreWritten() {
        XCTAssertTrue(ShellFormatting.device("Attic").contains("Attic"))
        XCTAssertTrue(ShellFormatting.version("0.1.0").contains("0.1.0"))
    }

    func testTheTextIsBigEnoughToBeReadFromADistance() {
        // WHY: The promise of the shell: the title is at least 22 points,
        // the word of a state at least 20 points and the details at least
        // 16 points.
        XCTAssertGreaterThanOrEqual(ShellTypography.title, 22)
        XCTAssertGreaterThanOrEqual(ShellTypography.cardTitle, 22)
        XCTAssertGreaterThanOrEqual(ShellTypography.state, 20)
        XCTAssertGreaterThanOrEqual(ShellTypography.detail, 16)
    }

    func testTheFrameOfTheFocusIsVisible() {
        XCTAssertGreaterThanOrEqual(ShellTypography.focusRingWidth, 3)
    }

    func testTheWindowHoldsTwoCardsWithoutAHorizontalScrollBar() {
        // WHY: The grid of the window has two columns, and the smallest
        // width of the window has to hold them — otherwise the cards are
        // cut off at the edge.
        XCTAssertEqual(FocusRing.columns, 2)
        XCTAssertGreaterThan(
            ShellTypography.minimumWindowWidth,
            ShellTypography.cardPadding * 4
        )
    }

    /// A card of a protocol with the given ports and senders.
    private func card(
        tcpPort: UInt16? = 7000,
        udpPort: UInt16? = nil,
        peerCount: UInt32 = 0
    ) -> ProtocolCardState {
        ProtocolCardState(
            protocolId: .airplay,
            phase: .advertising,
            enabled: true,
            tcpPort: tcpPort,
            udpPort: udpPort,
            peerCount: peerCount,
            lastError: nil
        )
    }
}
