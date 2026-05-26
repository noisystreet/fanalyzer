//! 组合页表单 localStorage 脚本（纯 SSR 注入）。

/// 返回注入组合页的 `<script>` 内容。
pub fn portfolio_draft_script() -> &'static str {
    r#"(function(){
  var KEY="fanalyzer.portfolio.draft";
  var form=document.querySelector('form.portfolio-form');
  if(!form)return;
  var params=new URLSearchParams(location.search);
  if(params.get('import')==='watchlist')return;
  if(!params.get('holdings')&&params.get('run')!=='1'){
    try{
      var raw=localStorage.getItem(KEY);
      if(raw){
        var d=JSON.parse(raw);
        var n=form.querySelector('[name=name]');
        var h=form.querySelector('[name=holdings]');
        if(n&&d.name)n.value=d.name;
        if(h&&d.holdings)h.value=d.holdings;
      }
    }catch(e){}
  }
  function save(){
    var n=form.querySelector('[name=name]');
    var h=form.querySelector('[name=holdings]');
    localStorage.setItem(KEY,JSON.stringify({
      name:n?n.value:'',
      holdings:h?h.value:''
    }));
  }
  form.addEventListener('input',save);
  form.addEventListener('change',save);
})();"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_script_contains_storage_key() {
        assert!(portfolio_draft_script().contains("fanalyzer.portfolio.draft"));
    }
}
