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
    minimumHeight: 400 + 2 * margin

    Item {
        id: container

        anchors.fill: parent
        anchors.margins: margin
    }

    statusBar: StatusBar {
        anchors.fill: parent
        RowLayout {
            anchors.fill: parent

            Label { text: ""; id: statusLabel; anchors.left: parent.left }
            Label { text: ""; id: logLabel; anchors.right: parent.right }
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

    property var connectScreen: Qt.createQmlObject(assets.get_connect_screen(), container, "connect_screen.qml")
    property var gameScreen: Qt.createQmlObject(assets.get_game_screen(), container, "game_screen.qml")
    property var lobbyScreen: Qt.createQmlObject(assets.get_lobby_screen(), container, "lobby_screen.qml")
    property var loginScreen: Qt.createQmlObject(assets.get_login_screen(), container, "login_screen.qml")

    Component.onCompleted: {
        // connect different screens
        connectScreen.connected.connect(function () {
            connectScreen.deactivate();
            loginScreen.activate();
        });

        loginScreen.disconnected.connect(function () {
            loginScreen.deactivate();
            connectScreen.activate();
        });

        loginScreen.registered.connect(function () {
            loginScreen.deactivate();
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
