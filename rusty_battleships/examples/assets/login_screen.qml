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
                text: "Please select a server from the list."
            }

            ColumnLayout {
                Layout.fillWidth: true

                TextField {
                    id: customServer
                    Layout.fillWidth: true

                    placeholderText: "Example: 127.0.0.1:5000"

                    property bool validIp: false

                    style: TextFieldStyle {
                        background: Rectangle {
                            radius: 2
                            border.color: customServer.validIp ? "black" : "red"
                            border.width: 1
                        }
                    }
                }

                RowLayout {
                    id: updateNotice

                    BusyIndicator {
                        implicitHeight: 10; implicitWidth: 10
                    }
                    Label {
                        text: "Logging in"
                    }
                }
            }

            Button {
                anchors.topMargin: 50
                text: "Register"

                onClicked: {
                    console.log("Yay!");
                }
            }
        }
    }


    // FIXME
    function login() {
        bridge.send_login_request(usernameField.text);
        bridge.poll_state();
    }

    // FIXME: remove
    function features() {
        bridge.send_get_features_request();
        bridge.poll_state();
        featuresLabel.text = bridge.get_features_list();
    }


    function activate() {
      timer.triggered.connect(updateServers);
      visible = true;
    }

    function deactivate() {
      timer.triggered.disconnect(updateServers);
      visible = false;
    }
}
