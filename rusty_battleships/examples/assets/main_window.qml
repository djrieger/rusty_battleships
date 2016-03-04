import QtQuick 2.2
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.0
import QtQuick.Dialogs 1.1

ApplicationWindow {
    visible: true
    title: "Battleships"

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
                rows: 10
                columns: 10
            }
        }

    ColumnLayout {
         RowLayout {
              TextField {
                id: hostnameField
                Layout.fillWidth: true

                placeholderText: "Enter host"
                focus: true

                onAccepted: connect()
              }

              Button {
                text: "Connect"

                onClicked: connect()
              }
          }

         RowLayout {
              TextField {
                id: usernameField
                Layout.fillWidth: true

                placeholderText: "Enter nickname"
                focus: true

                onAccepted: login()
              }

              Button {
                text: "Login"

                onClicked: login()
              }
            }

              Label {
                id: infoLabel
                text: ""
              }
    }

        ListView {
            id: userList
            width: 200
            Layout.fillHeight: true
            model: ListModel {
                ListElement {
                    name: "Grey"
                    colorCode: "grey"
                }

                ListElement {
                    name: "Red"
                    colorCode: "red"
                }

                ListElement {
                    name: "Blue"
                    colorCode: "blue"
                }

                ListElement {
                    name: "Green"
                    colorCode: "green"
                }
            }
            delegate: Item {
                x: 5
                width: 80
                height: 40
                Row {
                    id: row1
                    spacing: 10
                    Rectangle {
                        width: 40
                        height: 40
                        color: colorCode
                    }

                    Text {
                        text: name
                        anchors.verticalCenter: parent.verticalCenter
                        font.bold: true
                    }
                }
            }
        }
    }

    RowLayout {
        Rectangle {
            width: 200; height: 200; color: "black"

            Grid {
                x: 5; y: 5
                rows: 6; columns: 6; spacing: 2

                Repeater { 
                    model: 36
                    Button { 
                        width: 30; height: 30
                        Text { text: "X" //index
                          font.pointSize: 15
                          anchors.centerIn: parent 
                        }
                    }
                }
            }
        }

        Rectangle {
            width: 200; height: 200; color: "black"

            Grid {
                x: 5; y: 5
                rows: 6; columns: 6; spacing: 2

                Repeater { 
                    model: 36
                    Button { 
                        width: 30; height: 30
                        Text { text: "X" //index
                          font.pointSize: 15
                          anchors.centerIn: parent 
                        }
                    }
                }
            }
        }
    }

               function login() {
                  bridge.send_login_request(usernameField.text);
              }

              function connect() {
                  bridge.connect(hostnameField.text);
              }
}

