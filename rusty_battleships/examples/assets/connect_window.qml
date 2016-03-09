import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.0
import QtQuick.Dialogs 1.1

ApplicationWindow {
    visible: true
    title: "Verbinden"

    property int margin: 11
    minimumWidth: 400 + 2 * margin
    minimumHeight: 300 + 2 * margin

    Timer {
        interval: 50
        running: true
        repeat: true
        onTriggered: {
            // TODO: Read servers from bridge
        }
    }

    ColumnLayout {
        id: mainLayout
        /* anchors.fill: parent */
        anchors.margins: margin

        Label {
            id: infoLabel
            text: "Gefundene Server:"
        }

        ListView { 
            id: serverList
            width: 200
            height: 200
            /* Layout.fillHeight: true */
            model: ListModel {
                ListElement { name: "Server1" }
                ListElement { name: "Server2" }
                ListElement { name: "Server3" }
                ListElement { name: "Server4" }
            }
            delegate: Item {
                x: 5
                width: 80
                height: 15
                Row {
                    id: row1
                    spacing: 10
                    Rectangle {
                        width: 200
                        height: 15

                        Text {
                            text: name
                            anchors.verticalCenter: parent.verticalCenter
                            font.bold: true
                        }

                        MouseArea {
                            id: mouse_area1
                            z: 1
                            hoverEnabled: true
                            anchors.fill: parent

                            onClicked:{
                                serverList.currentIndex = index 
                                hostnameField.text = ""
                                portField.text = ""
                                console.log("Chose server " + index);
                            }
                        }
                    }
                }
            }
        }

        TextField {
            id: nicknameField
            Layout.fillWidth: true
            placeholderText: "Nickname"
            focus: true
        }

        RowLayout { 
            TextField {
                id: hostnameField
                Layout.fillWidth: true
                placeholderText: "Host"
            }

            TextField {
                id: portField
                Layout.fillWidth: true
                placeholderText: "Port"
            }
        }

        Button {
            text: "Verbinden"
            onClicked: connect()
        }
    }

    function connect() {
        if (hostnameField.text != "") {
            console.log("Connecting to " + hostnameField.text + ":" + portField.text);
            bridge.connect(hostnameField.text, portField.text, nicknameField.text);
        } else {
            console.log("Connecting to discovered server " + serverList.currentIndex);
            bridge.connect_to_discovered(serverList.currentIndex, nicknameField.text);
        }
    }
}
