import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.0
import QtQuick.Dialogs 1.1

ApplicationWindow {
    visible: true
    title: "Rusty Battleships v1.337"

    property int margin: 11
    width: mainLayout.implicitWidth + 2 * margin
    height: mainLayout.implicitHeight + 2 * margin
    minimumWidth: 800 + 2 * margin
    minimumHeight: 600 + 2 * margin

    Timer {
        interval: 50
        running: true
        repeat: true
        onTriggered: {
            statusLabel.text = bridge.poll_state();
            logLabel.text = bridge.poll_log();
        }
    }

    statusBar: StatusBar {
        RowLayout {
            anchors.fill: parent
            RowLayout {
                Label { text: "Read Only"; id: statusLabel }
                Label { text: "Read Only"; id: logLabel }
            }
        }
    }

    RowLayout {
        id: mainLayout
        anchors.fill: parent
        anchors.margins: margin

        Rectangle {
            id: fieldContainer
            color: "white"
            Layout.fillWidth: true
            Layout.fillHeight: true

            GridLayout {
                id: field
                anchors.centerIn: parent
                width: Math.min(fieldContainer.height, fieldContainer.width)
                height: width
                rows: 5
                columns: 5
            }
        }

        // ColumnLayout {
            // Label {
            //     id: infoLabel
            //     text: ""
            // }
        // }

        ListView { //TODO: Needs to be filled.
            id: userList
            width: 200
            Layout.fillHeight: true
            model: ListModel {
                ListElement {
                    name: "Captain Kirk"
                    colorCode: "lightgrey"
                }

                ListElement {
                    name: "Captain Nemo"
                    colorCode: "lime"
                }

                ListElement {
                    name: "Admiral Ackbar"
                    colorCode: "lightgrey"
                }

                ListElement {
                    name: "Captain Balou"
                    colorCode: "lightgrey"
                }
            }
            delegate: Item {
                x: 5
                width: 80
                height: 15
                Row {
                    id: row1
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 3

                    Rectangle {
                        width: 200
                        height: 15
                        color: "transparent"

                        Rectangle {
                            width: 15
                            height: 15
                            color: colorCode
                            anchors.left: parent.left

                            Text {
                                text: name
                                anchors.left: parent.right
                                font.bold: false
                            }
                        }

                        MouseArea {
                            id: mouse_area1
                            z: 1
                            hoverEnabled: true
                            anchors.fill: parent

                            onClicked:{
                                userList.currentIndex = index
                                console.log("Challenged player " + index);
                            }
                        }
                    }
                }
            }
        }
    }

    RowLayout {
        Rectangle {
            width: 200; height: 200; color: "blue"

            Grid {
                x: 5; y: 5
                rows: 10; columns: 10; spacing: 1

                Repeater {
                    model: 100
                    Button {
                        width: 18; height: 18
                        Text { text: "X" //index
                            font.pointSize: 10
                            anchors.centerIn: parent
                        }
                    }
                }
            }
        }

        Rectangle {
            width: 200; height: 200; color: "blue"

            Grid {
                x: 5; y: 5
                rows: 10; columns: 10; spacing: 1

                Repeater {
                    model: 100
                    Button {
                        width: 18; height: 18
                        Text { text: "X" //index
                            font.pointSize: 10
                            anchors.centerIn: parent
                        }
                    }
                }
            }
        }
    }

    function login() {
        bridge.send_login_request(usernameField.text);
        bridge.poll_state();
    }

    function features() {
        bridge.send_get_features_request();
        bridge.poll_state();
        featuresLabel.text = bridge.get_features_list();
    }

    function update_status() {
        Statusbar.statusLabel = bridge.get_last_message();
    }

    function connect() {
        bridge.connect(hostnameField.text);
    }

    function update_lobby() {
        bridge.get_ready_players();
        bridge.get_available_players();
        //^-- verwursten in list items!
    }
}
