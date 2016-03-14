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
        Label {
          id: statusMessage;
          anchors.left: parent.left
        }
        Label {
          id: logMessage;
          anchors.right: parent.right
        }
    }

    Timer {
        id: timer

        interval: 250
        running: true
        repeat: true

        onTriggered: {
            statusMessage.text = bridge.poll_state();
            logMessage.text = bridge.poll_log();

            if (bridge.connection_closed()) {
                if (gameScreen.visible) {
                    gameScreen.deactivate();
                    connectScreen.activate();
                } else if (lobbyScreen.visible) {
                    lobbyScreen.deactivate();
                    connectScreen.activate();
                } else if (loginScreen.visible) {
                    loginScreen.deactivate();
                    connectScreen.activate();
                }
            }
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
