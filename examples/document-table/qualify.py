#!/usr/bin/env python3
"""Real app-server main and child use host tools; domain conversion stays in spools."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys
import time
from uuid import uuid4

from prepare import prepare


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('host', 'host-tests', 'loom', 'codex', 'python', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    sys.path.insert(0, str(args.host_tests.resolve(strict=True)))
    from app_server_support import Driver, sha
    from host_qualification import require_native_workspace
    require_native_workspace()
    from reportlab.pdfgen.canvas import Canvas
    from openpyxl import load_workbook
    host, loom, binary = [p.resolve(strict=True) for p in (args.host, args.loom, args.codex)]
    python = args.python.absolute()
    driver = Driver(host, args.output)
    driver.expires = int(time.time()*1000) + 900000
    try:
        root = driver.root
        pdf = root / 'source.pdf'
        canvas = Canvas(str(pdf), pagesize=(500, 360))
        text = canvas.beginText(35, 310)
        for line in ['Parts order', '| Item | Quantity | Unit price |', '| --- | --- | --- |',
                     '| Bolt | 12 | 0.5 |', '| Plate | 3 | 8 |', '| Bearing | 2 | 4.25 |']:
            text.textLine(line)
        canvas.drawText(text)
        canvas.save()
        first, second = prepare(loom, host, python, root)
        for instance in (first, second): driver.request({'op':'register','instance':str(instance)})
        def resource(instance, name):
            return {'instance':str(instance),'run':'convert','output':name}
        pdf_right = {'kind':'read-input','path':str(pdf),'sha256':sha(pdf)}
        md_right = dict(resource(first,'document.md'),kind='artifact')
        permissions = [pdf_right, md_right, dict(resource(second,'result.xlsx'),kind='artifact'),
            {'kind':'run','instance':str(first),'run':'convert','input':str(pdf),'seconds':30},
            {'kind':'accept','instance':str(first),'run':'convert'},
            {'kind':'accept','instance':str(second),'run':'convert'}]
        # The downstream run permission needs an exact artifact revision. Its immutable
        # bytes and provenance do not exist until the first stage is accepted.
        instructions = ('Use host_request to execute the exact listed operation. For mutations call preview, then the same request with approve from that preview. '
                        'Poll task until state is settled and outcome=succeeded, then accept. If any stage fails return ok=false immediately. Do not repeat submit after an error. ')
        first_task = driver.task(binary, instructions + json.dumps({'op':'submit','instance':str(first),'run':'convert','input':str(pdf),'seconds':30})+
            '\nAccept the completed run with '+json.dumps({'op':'accept','instance':str(first),'run':'convert'})+'. Return ok true after success.',scope={'stage':'extract'})
        parent = driver.grant(permissions + [driver.profile(first_task)],delegate=True)
        task_id = driver.approve({'op':'harness','task':first_task}, parent)['id']
        driver.wait(task_id,parent)
        assert driver.request({'op':'harness-result','id':task_id},parent)['proposal']=={'ok':True}
        first_report = driver.call('spool-artifacts',first,'convert','--output','document.md')
        revision = first_report['output']['binding']['revision']
        catalog = first_report['catalog']
        md = driver.request({'op':'artifact-read',**resource(first,'document.md')},parent)
        (root / 'document.md').write_bytes(bytes(md['bytes']))
        assert all(w in bytes(md['bytes']).decode() for w in ('Bolt','Plate','Bearing'))
        run_second = {'op':'submit','instance':str(second),'run':'convert','artifact':{'catalog':catalog,'revision':revision},'seconds':30}
        second_right = {'kind':'run-artifact','instance':str(second),'run':'convert','catalog':catalog,'revision':revision,'seconds':30}
        child_schema = {'type':'object','properties':{'total':{'type':'number'},'source_denied':{'type':'boolean'}},'required':['total','source_denied'],'additionalProperties':False}
        child_prompt = ('Use host_request to try '+json.dumps({'op':'read-input','path':str(pdf),'sha256':sha(pdf)})+
            '. Expect denial. Then read '+json.dumps({'op':'artifact-read',**resource(first,'document.md')})+
            '. Decode the UTF-8 byte array and compute the sum of quantity times unit price. Return total and source_denied. You receive no parent conversation.')
        child_task = driver.task(binary,child_prompt,scope={'stage':'review','markdown_revision':revision},schema=child_schema)
        child_profile = driver.profile(child_task)
        parent_secret = 'parent-only-' + uuid4().hex
        second_task = driver.task(binary,'placeholder',scope={'stage':'finish','markdown_revision':revision})
        # A new explicitly approved scope adds the now-existing fixed revision.
        second_parent = driver.grant(permissions + [second_right, child_profile,driver.profile(second_task)],subject='main-fixed-revision',delegate=True)
        child = driver.grant([md_right,child_profile],subject='review-subagent',parent=second_parent)
        child_request = {'op':'delegate','parent':'document-study','grant':child['id'],'task':child_task}
        finish_prompt = (instructions + json.dumps(run_second)+'\nAccept it with '+json.dumps({'op':'accept','instance':str(second),'run':'convert'})+
            '\nThen delegate the bounded review using '+json.dumps(child_request)+
            '. Poll the returned task using the same child grant reference. Read its harness-result with that grant and require total=38.5 and source_denied=true. '
            'Return ok=true only after the accepted workbook and successful child review.\nPrivate main marker, never disclose to child: '+parent_secret)
        second_task['prompt'] = finish_prompt
        main_job = driver.approve({'op':'harness','task':second_task},second_parent)['id']
        driver.wait(main_job,second_parent)
        assert driver.request({'op':'harness-result','id':main_job},second_parent)['proposal']=={'ok':True}
        final = driver.request({'op':'artifact-read',**resource(second,'result.xlsx')},second_parent)
        workbook = root / 'result.xlsx'
        workbook.write_bytes(bytes(final['bytes']))
        rows = list(load_workbook(workbook,data_only=True).active.values)
        assert rows[1:4] == [('Bolt',12,.5,6),('Plate',3,8,24),('Bearing',2,4.25,8.5)] and rows[-1][-1] == 38.5
        children = [p for p in (driver.service/'jobs').iterdir() if json.loads((p/'request.json').read_text()).get('authorization',{}).get('id')==child['id']]
        assert len(children)==1
        child_dir = children[0]
        child_result = driver.request({'op':'harness-result','id':child_dir.name},child)['proposal']
        assert child_result == {'total':38.5,'source_denied':True}
        assert parent_secret not in (child_dir/'request.json').read_text() and parent_secret not in (child_dir/'native-events.jsonl').read_text()
        driver.request({'op':'harness-result','id':main_job},child,ok=False)
        driver.request({'op':'derive-grant','scope':{'task':'document-study','subject':'escalation','expires_ms':driver.expires,
            'delegate':True,'permissions':[pdf_right]}},child,ok=False)
        # Revoke and cancel real active model turns after their start notification.
        cancelled = []
        for action in ('cancel','revoke'):
            hold_task = dict(child_task,prompt='Call host_request with '+json.dumps({'op':'artifact-read',**resource(first,'document.md')})+'. Then carefully calculate and verify each row before answering.')
            leaf = driver.grant([md_right,child_profile],subject=action,parent=second_parent)
            job = driver.approve({'op':'harness','task':hold_task},leaf)['id']
            driver.ready(job)
            if action=='cancel': driver.approve({'op':'cancel','id':job},second_parent)
            else: driver.approve({'op':'revoke-grant','id':leaf['id']},second_parent)
            driver.wait(job,leaf,'cancelled')
            completion=json.loads((driver.service/'jobs'/job/'completion.json').read_text())['result']
            assert completion['termination_confirmed'] and not completion['proposal_available']
            cancelled.append(job)
        driver.approve({'op':'revoke-grant','id':second_parent['id']},second_parent)
        driver.request({'op':'artifact-read',**resource(first,'document.md')},child,ok=False)
        assert driver.call('spool-artifacts',second,'convert','--output','result.xlsx')['output']['binding']['content_sha256']==sha(workbook)
        model_threads = []
        for job in (task_id, main_job, child_dir.name):
            events = [json.loads(line) for line in (driver.service/'jobs'/job/'native-events.jsonl').read_text().splitlines()]
            start = next(event['result']['thread'] for event in events if event.get('id') == 1 and 'result' in event)
            finish = next(event['params']['turn'] for event in events if event.get('method') == 'turn/completed')
            model_threads.append({'model': start.get('model'), 'duration_ms': finish.get('durationMs')})
        child_receipts = [json.loads(line) for line in (child_dir/'tool-receipts.jsonl').read_text().splitlines()]
        assert any(not receipt['succeeded'] for receipt in child_receipts)
        summary={'passed':True,'real_codex_app_server':True,'host_sha256':sha(host),'codex_sha256':sha(binary),
            'source_pdf_sha256':sha(pdf),'markdown_revision':revision,'xlsx_sha256':sha(workbook),'total':38.5,
            'models':model_threads,'main_attempts':[task_id,main_job],'child_attempt':child_dir.name,'cancelled_attempts':cancelled,
            'child_source_denied':True,'child_context_isolated':True,'revocation_preserves_products':True,'automatic_retries':0}
        (root/'acceptance.json').write_text(json.dumps(summary,indent=2))
        print(json.dumps(summary))
    finally:
        driver.close()


if __name__=='__main__': main()
