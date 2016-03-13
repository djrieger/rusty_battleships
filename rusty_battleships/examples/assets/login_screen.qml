import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Controls.Styles 1.2
import QtQuick.Layouts 1.0
import QtQuick.Dialogs 1.1

Item {
    id: screen

    anchors.fill: parent
    visible: false

    // TODO: provide button
    signal disconnected();
    signal registered();

    GroupBox {
        anchors.left: parent.left
        anchors.right: parent.right

        title: "Choose a nickname"

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 5
            spacing: 12

            Label {
                text: "Please enter the nickname you would like to use."
            }

            ColumnLayout {
                Layout.fillWidth: true

                TextField {
                    id: nickname
                    Layout.fillWidth: true

                    placeholderText: "Example: john_doe"

                    style: TextFieldStyle {
                        background: Rectangle {
                            radius: 2
                            border.color: nickname.acceptableInput ? "black" : "red"
                            border.width: 1
                        }
                    }

                    validator: RegExpValidator { regExp: /^[\x21-\x7E]{1,255}$/ }
                }

                RowLayout {
                    id: progressNotice

                    visible: false

                    BusyIndicator {
                        implicitHeight: 10; implicitWidth: 10
                    }
                    Label {
                        text: "Logging in"
                    }
                }
            }

            RowLayout {
                Button {
                    anchors.topMargin: 50
                    enabled: nickname.acceptableInput && !registering
                    text: "Register"

                    property bool registering: false

                    onClicked: {
                        registering = true;
                        progressNotice.visible = true
                        nicknameError.visible = false

                        if (bridge.send_login_request(nickname.text)) {
                            screen.registered();
                        } else {
                            nicknameError.visible = true
                        }

                        progressNotice.visible = false
                        registering = false;
                    }
                }
                Label {
                    id: nicknameError

                    color: "red"
                    text: "Nickname already taken, please select a different one"
                    visible: false
                }
            }
        }
    }


    // FIXME
    function login() {
    }


    function activate() {
      visible = true;
    }

    function deactivate() {
      visible = false;
    }
}
