import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.0
import QtQuick.Dialogs 1.1

ApplicationWindow {
    id: window

    visible: true
    title: "Rusty Battleships"

    property int margin: 11
    minimumWidth: 400 + 2 * margin
    minimumHeight: 300 + 2 * margin

    property var connectScreen: Qt.createQmlObject(assets.get_connect_screen(), window, "connect_screen.qml")
    property var gameScreen: Qt.createQmlObject(assets.get_game_screen(), window, "game_screen.qml")
    property var lobbyScreen: Qt.createQmlObject(assets.get_lobby_screen(), window, "lobby_screen.qml")

    statusBar: StatusBar {
        RowLayout {
            anchors.fill: parent
            RowLayout {
                Label { text: "Read Only"; id: statusLabel }
                Label { text: "Read Only"; id: logLabel }
            }
        }
    }

    Timer {
        id: timer

        interval: 500
        running: true
        repeat: true

        onTriggered: {
            // FIXME: less debugg-y messages
            statusLabel.text = bridge.poll_state();
            logLabel.text = bridge.poll_log();
        }
    }

    Component.onCompleted: {
        // connect different screens
        connectScreen.connected.connect(function () {
            connectScreen.deactivate();
            lobbyScreen.activate();
        });

        lobbyScreen.disconnected.connect(function () {
            lobbyScreen.deactivate();
            connectScreen.activate();
        });

        lobbyScreen.gameStarted.connect(function () {
            lobbyScreen.deactivate();
            gameScreen.activate();
        });

        gameScreen.gameEnded.connect(function () {
            gameScreen.deactivate();
            lobbyScreen.activate();
        });

        // activate initial screen
        connectScreen.activate();
    }
}
