//simple HTTP server for node.js

const {InfluxDB, Point} = require('@influxdata/influxdb-client')

const token = "<InfluxDB Token>"
const url = 'http://<PC Address>:8086'

const client = new InfluxDB({url, token})
let org = `<ORGANIZATION NAME>`
let bucket = `LOGGER`

let writeClient = client.getWriteApi(org, bucket, 'ns')


const http = require('http')
const port = 3000
// respond to GET
const respondGet = (url, res) => {
    const fs = require('fs')
    const filename = (url == '/') ? 'index.html' : '.' + decodeURIComponent(url)
    fs.readFile(filename, function(err, data){
        res.statusCode = 200
        res.end(data)
    })
}

// respond to POST
let i = 0
let timestamp = 9999999999;
let diff_start_time = 0;
let start_time = 0;
const respondPost = (req, res) => {
    let posted = ''
    req.on('data', (chunk) => {
        posted += chunk
    }).on('end', () => {
        try {
            console.log(posted)
            let json = JSON.parse(posted)
            // console.log(json)
            for (const it of json) {
                if (timestamp > it.timestamp) {
                    diff_start_time = it.timestamp
                    start_time = Date.now()
                }
                timestamp = it.timestamp
                if (it.measurement == "heatpanel"){
                    let point = new Point(it.measurement)
                        .tag('tag', it.tag)
                        .floatField('PT', it.PT)
                        .floatField('TT', it.TT)
                        .timestamp(new Date(start_time + it.timestamp - diff_start_time))

                    writeClient.writePoint(point)
                }
                else {
                    let point = new Point(it.measurement)
                    .tag('tag', it.tag)
                    .floatField('temp', it.temp)
                    .floatField('bat', it.bat)
                    .timestamp(new Date(start_time + it.timestamp - diff_start_time))

                    writeClient.writePoint(point)
                }
                i = i + 1
            }
            writeClient.flush()
            res.end(JSON.stringify({ DATA: i }))
        } catch (err) {
            res.end(JSON.stringify({ message: 'server error' }))
        }
    })
}

// creating server
const server = http.createServer((req, res) => {
    switch (req.method) {
    case 'GET':
        respondGet(req.url, res)
        break
    case 'POST': 
        respondPost(req, res)
        break
    }
})

server.listen(port, () => {
    console.log('Server running')
})
