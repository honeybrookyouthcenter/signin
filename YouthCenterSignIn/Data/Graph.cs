using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.Graph;
using Microsoft.Toolkit.Services.MicrosoftGraph;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn
{
    class Graph
    {
        const string AppId = "6cd2d40f-e8d2-46be-ad3f-ac087d4b5284";
        readonly string[] Scopes = { "User.Read", "Contacts.ReadWrite" };

        public Graph()
        {
            MicrosoftGraphService instance = MicrosoftGraphService.Instance;
            instance.AuthenticationModel = MicrosoftGraphEnums.AuthenticationModel.V2;
            instance.Initialize(AppId,
                MicrosoftGraphEnums.ServicesToInitialize.UserProfile,
                Scopes);
        }

        public bool IsAuthenticated => MicrosoftGraphService.Instance.IsAuthenticated;

        public GraphServiceClient Provider => MicrosoftGraphService.Instance.GraphProvider;

        public async Task<bool> Login() => await MicrosoftGraphService.Instance.TryLoginAsync();

        #region Contacts

        public async Task<IEnumerable<Contact>> GetContacts()
        {
            if (!IsAuthenticated)
            {
                await ((UwpDataProvider)DataProvider.Current).ShowAuthenticationMessage();
                return new List<Contact>();
            }

            var contactFolder = await GetContactFolder();
            return contactFolder.Contacts;
            //    .Select("id,givenName,surname,birthday,homeAddress")
            //    .GetAsync();
        }

        public async Task SaveContact(Contact contact)
        {
            throw new NotImplementedException();
            //var contactStore = await ContactManager.RequestStoreAsync(ContactStoreAccessType.AppContactsReadWrite);
            //ContactList contactList = await GetContactList(contactStore);
            //await contactList.SaveContactAsync(contact);
        }

        #endregion

        #region Contact folders

        const string YouthCenterFolderId = "Youth Center";

        async Task<ContactFolder> GetContactFolder()
        {
            var folders = await Provider.Me.ContactFolders.Request()
                .Filter($"displayName eq '{YouthCenterFolderId}'")
                .Select("id")
                .Expand("contacts($select=id,givenName,surname,birthday,homeAddress)")
                .Top(1)
                .GetAsync();

            return folders.FirstOrDefault() ?? await CreateContactFolder();
        }

        async Task<ContactFolder> CreateContactFolder()
        {
            var newFolder = new ContactFolder { DisplayName = YouthCenterFolderId };
            var folder = await Provider.Me.ContactFolders.Request()
                .Select("id")
                .AddAsync(newFolder);
            return folder;
        }

        #endregion
    }
}
