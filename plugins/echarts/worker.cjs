// Protocol stdout contains JSON lines only. Diagnostics belong on stderr.
const echarts = require('echarts');
const readline = require('node:readline');
let chart, option;
function render() { return chart.renderToSVGString(); }
function handle(req) {
  if (req.component !== 'chart') throw new Error('Unknown component');
  switch (req.op) {
    case 'mount':
      chart = echarts.init(null, null, { renderer: 'svg', ssr: true, width: req.width, height: req.height });
      option = req.props.option;
      chart.setOption({...option, animation:false}, {notMerge:true});
      return {svg:render()};
    case 'update':
      option = req.props.option;
      chart.resize({width:req.width,height:req.height});
      chart.setOption({...option,animation:false}, {notMerge:true});
      return {svg:render()};
    case 'resize':
      chart.resize({width:req.width,height:req.height});
      return {svg:render()};
    case 'call':
      if (req.method === 'getOption') return {result:chart.getOption()};
      if (req.method === 'dispatchAction') {chart.dispatchAction(req.args); return {result:true,svg:render()};}
      throw new Error('Unknown method');
    case 'input': {
      // Use ECharts display objects for hit testing in SSR (no browser DOM).
      const list = chart.getZr().storage.getDisplayList(true);
      for (let i=list.length-1;i>=0;i--) {
        const el=list[i];
        if (el.ignore || el.invisible || !el.contain(req.event.x,req.event.y)) continue;
        const data = echarts.helper.getECData(el);
        if (data.dataIndex == null || data.seriesIndex == null) continue;
        const series = chart.getModel().getSeriesByIndex(data.seriesIndex);
        const params = series.getDataParams(data.dataIndex);
        const value = Array.isArray(params.value) ? params.value.at(-1) : params.value;
        if (typeof value !== 'number') continue;
        return {events:[{name:'click',payload:{seriesIndex:data.seriesIndex,dataIndex:data.dataIndex,name:String(params.name),value}}]};
      }
      return {};
    }
    case 'unmount': if (chart) chart.dispose(); chart=null; return {};
    default: throw new Error('Unknown operation');
  }
}
readline.createInterface({input:process.stdin,crlfDelay:Infinity}).on('line', line => {
  try {process.stdout.write(JSON.stringify(handle(JSON.parse(line)))+'\n');}
  catch(e) {process.stdout.write(JSON.stringify({error:e.message})+'\n');}
});
